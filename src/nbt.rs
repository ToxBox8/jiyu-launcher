//! NBT sin comprimir (el formato de `servers.dat`). Se lee entero y se vuelve a escribir igual,
//! así no se pierde nada de lo que Minecraft guarde ahí (iconos, «hidden», «acceptTextures»…).

#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(u8, Vec<Tag>),
    Compound(Vec<(String, Tag)>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl Tag {
    fn id(&self) -> u8 {
        match self {
            Tag::Byte(_) => 1,
            Tag::Short(_) => 2,
            Tag::Int(_) => 3,
            Tag::Long(_) => 4,
            Tag::Float(_) => 5,
            Tag::Double(_) => 6,
            Tag::ByteArray(_) => 7,
            Tag::String(_) => 8,
            Tag::List(..) => 9,
            Tag::Compound(_) => 10,
            Tag::IntArray(_) => 11,
            Tag::LongArray(_) => 12,
        }
    }

    pub fn campo(&self, nombre: &str) -> Option<&Tag> {
        match self {
            Tag::Compound(v) => v.iter().find(|(k, _)| k == nombre).map(|(_, t)| t),
            _ => None,
        }
    }

    pub fn texto(&self) -> Option<&str> {
        match self {
            Tag::String(s) => Some(s),
            _ => None,
        }
    }
}

struct Lector<'a> {
    d: &'a [u8],
    i: usize,
}

impl<'a> Lector<'a> {
    fn toma(&mut self, n: usize) -> Option<&'a [u8]> {
        let r = self.d.get(self.i..self.i + n)?;
        self.i += n;
        Some(r)
    }
    fn u8(&mut self) -> Option<u8> {
        Some(self.toma(1)?[0])
    }
    fn i16(&mut self) -> Option<i16> {
        Some(i16::from_be_bytes(self.toma(2)?.try_into().ok()?))
    }
    fn i32(&mut self) -> Option<i32> {
        Some(i32::from_be_bytes(self.toma(4)?.try_into().ok()?))
    }
    fn i64(&mut self) -> Option<i64> {
        Some(i64::from_be_bytes(self.toma(8)?.try_into().ok()?))
    }
    fn texto(&mut self) -> Option<String> {
        let n = self.i16()? as u16 as usize;
        Some(String::from_utf8_lossy(self.toma(n)?).to_string())
    }
    fn carga(&mut self, id: u8, prof: u32) -> Option<Tag> {
        if prof > 64 {
            return None;
        }
        Some(match id {
            1 => Tag::Byte(self.u8()? as i8),
            2 => Tag::Short(self.i16()?),
            3 => Tag::Int(self.i32()?),
            4 => Tag::Long(self.i64()?),
            5 => Tag::Float(f32::from_bits(self.i32()? as u32)),
            6 => Tag::Double(f64::from_bits(self.i64()? as u64)),
            7 => {
                let n = self.i32()?.max(0) as usize;
                Tag::ByteArray(self.toma(n)?.iter().map(|b| *b as i8).collect())
            }
            8 => Tag::String(self.texto()?),
            9 => {
                let t = self.u8()?;
                let n = self.i32()?.max(0) as usize;
                let mut v = Vec::with_capacity(n.min(4096));
                for _ in 0..n {
                    v.push(self.carga(t, prof + 1)?);
                }
                Tag::List(t, v)
            }
            10 => {
                let mut v = Vec::new();
                loop {
                    let t = self.u8()?;
                    if t == 0 {
                        break;
                    }
                    let k = self.texto()?;
                    v.push((k, self.carga(t, prof + 1)?));
                }
                Tag::Compound(v)
            }
            11 => {
                let n = self.i32()?.max(0) as usize;
                Tag::IntArray((0..n).map(|_| self.i32()).collect::<Option<_>>()?)
            }
            12 => {
                let n = self.i32()?.max(0) as usize;
                Tag::LongArray((0..n).map(|_| self.i64()).collect::<Option<_>>()?)
            }
            _ => return None,
        })
    }
}

/// Lee un archivo NBT: devuelve el nombre de la raíz y su contenido.
pub fn lee(datos: &[u8]) -> Option<(String, Tag)> {
    let mut l = Lector { d: datos, i: 0 };
    if l.u8()? != 10 {
        return None;
    }
    let nombre = l.texto()?;
    Some((nombre, l.carga(10, 0)?))
}

fn pon_texto(v: &mut Vec<u8>, s: &str) {
    let b = s.as_bytes();
    v.extend_from_slice(&(b.len().min(u16::MAX as usize) as u16).to_be_bytes());
    v.extend_from_slice(&b[..b.len().min(u16::MAX as usize)]);
}

fn escribe_tag(v: &mut Vec<u8>, t: &Tag) {
    match t {
        Tag::Byte(x) => v.push(*x as u8),
        Tag::Short(x) => v.extend_from_slice(&x.to_be_bytes()),
        Tag::Int(x) => v.extend_from_slice(&x.to_be_bytes()),
        Tag::Long(x) => v.extend_from_slice(&x.to_be_bytes()),
        Tag::Float(x) => v.extend_from_slice(&x.to_bits().to_be_bytes()),
        Tag::Double(x) => v.extend_from_slice(&x.to_bits().to_be_bytes()),
        Tag::ByteArray(a) => {
            v.extend_from_slice(&(a.len() as i32).to_be_bytes());
            v.extend(a.iter().map(|b| *b as u8));
        }
        Tag::String(s) => pon_texto(v, s),
        Tag::List(id, l) => {
            v.push(if l.is_empty() { *id } else { l[0].id() });
            v.extend_from_slice(&(l.len() as i32).to_be_bytes());
            for x in l {
                escribe_tag(v, x);
            }
        }
        Tag::Compound(c) => {
            for (k, x) in c {
                v.push(x.id());
                pon_texto(v, k);
                escribe_tag(v, x);
            }
            v.push(0);
        }
        Tag::IntArray(a) => {
            v.extend_from_slice(&(a.len() as i32).to_be_bytes());
            for x in a {
                v.extend_from_slice(&x.to_be_bytes());
            }
        }
        Tag::LongArray(a) => {
            v.extend_from_slice(&(a.len() as i32).to_be_bytes());
            for x in a {
                v.extend_from_slice(&x.to_be_bytes());
            }
        }
    }
}

pub fn escribe(nombre: &str, raiz: &Tag) -> Vec<u8> {
    let mut v = vec![10];
    pon_texto(&mut v, nombre);
    escribe_tag(&mut v, raiz);
    v
}

#[cfg(test)]
mod pruebas {
    /// Lee los servers.dat de verdad (solo lectura) y comprueba que al reescribirlos salen idénticos.
    #[test]
    fn ida_y_vuelta_identica() {
        let raiz = std::env::var("HOME").unwrap() + "/.var/app/org.prismlauncher.PrismLauncher/data/PrismLauncher/instances";
        let mut vistos = 0;
        for e in std::fs::read_dir(raiz).into_iter().flatten().flatten() {
            let f = e.path().join("minecraft/servers.dat");
            let Ok(d) = std::fs::read(&f) else { continue };
            let (n, t) = super::lee(&d).expect("se lee");
            assert_eq!(super::escribe(&n, &t), d, "{f:?} cambia al reescribirlo");
            vistos += 1;
        }
        println!("servers.dat comprobados: {vistos}");
    }
}
