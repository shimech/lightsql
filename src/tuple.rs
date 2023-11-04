use crate::memcmpable;

pub fn encode(elems: impl Iterator<Item = impl AsRef<[u8]>>, bytes: &mut Vec<u8>) {
    elems.for_each(|elem| {
        let elem_bytes = elem.as_ref();
        let len = memcmpable::encoded_size(elem_bytes.len());
        bytes.reserve(len);
        memcmpable::encode(elem_bytes, bytes);
    })
}

pub fn decode(bytes: &[u8], elems: &mut Vec<Vec<u8>>) {
    let mut rest = bytes;
    while !rest.is_empty() {
        let mut elem = vec![];
        memcmpable::decode(&mut rest, &mut elem);
        elems.push(elem);
    }
}
