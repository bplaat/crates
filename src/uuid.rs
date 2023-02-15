use getrandom;

pub struct Uuid([u8; 16]);

impl Uuid {
    pub fn new() -> Uuid {
        let mut bytes = [0; 16];
        _ = getrandom::getrandom(&mut bytes);
        bytes[6] = bytes[6] & 0x0f | 0x40;
        bytes[8] = bytes[8] & 0x3f | 0x80;
        Uuid(bytes)
    }
}

const HEXS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

impl ToString for Uuid {
    fn to_string(&self) -> String {
        let mut sb = String::with_capacity(20);
        for i in 0..16 {
            sb.push(HEXS[(self.0[i] >> 4) as usize]);
            sb.push(HEXS[(self.0[i] & 15) as usize]);
            if i == 3 || i == 5 || i == 7 || i == 9 {
                sb.push('-');
            }
        }
        sb
    }
}
