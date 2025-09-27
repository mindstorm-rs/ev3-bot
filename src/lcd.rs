use core::{cell::Cell, marker::PhantomData};

use ev3rt::{lcd_apply, LCD_FRAMEBUFFER_ROWS, LCD_FRAMEBUFFER_ROW_BYTES};

pub const GLYPH_SIZE: usize = 18;
pub type GlyphSourceRow = u32;
pub struct GlyphSource {
    pub bits: [GlyphSourceRow; GLYPH_SIZE],
}

pub struct GlyphDataRowByte {
    pub byte: u8,
}
pub const GLYPH_DATA_ROW_BYTES: usize = GLYPH_SIZE / 3;
pub const LCD_GLYPHS_PER_ROW: usize = LCD_FRAMEBUFFER_ROW_BYTES / GLYPH_DATA_ROW_BYTES;
pub const LCD_GLYPHS_PER_COLUMN: usize = LCD_FRAMEBUFFER_ROWS / GLYPH_SIZE;
pub type GlyphDataRow = [u8; GLYPH_DATA_ROW_BYTES];
pub type GlyphData = [GlyphDataRow; GLYPH_SIZE];

impl GlyphDataRowByte {
    pub const fn new_direct((bit1, bit2, bit3): (u8, u8, u8)) -> Self {
        let mut byte = 0;
        byte |= (bit1 << 2) | (bit1 << 3);
        byte |= (bit2 << 4) | (bit2 << 5);
        byte |= (bit3 << 6) | (bit3 << 7);
        Self { byte }
    }

    pub const fn new_mirror((bit1, bit2, bit3): (u8, u8, u8)) -> Self {
        let mut byte = 0;
        byte |= (bit3 << 2) | (bit3 << 3);
        byte |= (bit2 << 4) | (bit2 << 5);
        byte |= (bit1 << 6) | (bit1 << 7);
        Self { byte }
    }
}

impl GlyphSource {
    pub const fn new(bits: [GlyphSourceRow; GLYPH_SIZE]) -> Self {
        Self { bits }
    }

    pub const fn bit(&self, row: usize, column: usize) -> u8 {
        ((self.bits[row] & (1 << column)) >> column) as u8
    }

    pub const fn row_bits_direct(&self, row: usize, byte_index: usize) -> (u8, u8, u8) {
        let base_column_index = byte_index * 3;
        let bit1 = self.bit(row, base_column_index + 0);
        let bit2 = self.bit(row, base_column_index + 1);
        let bit3 = self.bit(row, base_column_index + 2);
        (bit1, bit2, bit3)
    }

    pub const fn row_bits_mirror(&self, row: usize, byte_index: usize) -> (u8, u8, u8) {
        let byte_index = GLYPH_DATA_ROW_BYTES - (byte_index + 1);
        self.row_bits_direct(row, byte_index)
    }

    pub const fn column_bits_direct(&self, column: usize, byte_index: usize) -> (u8, u8, u8) {
        let base_row_index = byte_index * 3;
        let bit1 = self.bit(base_row_index + 0, column);
        let bit2 = self.bit(base_row_index + 1, column);
        let bit3 = self.bit(base_row_index + 2, column);
        (bit1, bit2, bit3)
    }

    pub const fn column_bits_mirror(&self, column: usize, byte_index: usize) -> (u8, u8, u8) {
        let byte_index = GLYPH_DATA_ROW_BYTES - (byte_index + 1);
        self.column_bits_direct(column, byte_index)
    }

    pub const fn row_byte_direct(&self, row: usize, byte_index: usize) -> u8 {
        let bits = self.row_bits_direct(row, byte_index);
        GlyphDataRowByte::new_direct(bits).byte
    }
    pub const fn row_byte_mirror(&self, row: usize, byte_index: usize) -> u8 {
        let bits = self.row_bits_mirror(row, byte_index);
        GlyphDataRowByte::new_mirror(bits).byte
    }

    pub const fn column_byte_direct(&self, column: usize, byte_index: usize) -> u8 {
        let bits = self.column_bits_direct(column, byte_index);
        GlyphDataRowByte::new_direct(bits).byte
    }
    pub const fn column_byte_mirror(&self, column: usize, byte_index: usize) -> u8 {
        let bits = self.column_bits_mirror(column, byte_index);
        GlyphDataRowByte::new_mirror(bits).byte
    }

    pub const fn row_direct(&self, row: usize) -> GlyphDataRow {
        [
            self.row_byte_direct(row, 0),
            self.row_byte_direct(row, 1),
            self.row_byte_direct(row, 2),
            self.row_byte_direct(row, 3),
            self.row_byte_direct(row, 4),
            self.row_byte_direct(row, 5),
        ]
    }
    pub const fn row_mirror(&self, row: usize) -> GlyphDataRow {
        [
            self.row_byte_mirror(row, 0),
            self.row_byte_mirror(row, 1),
            self.row_byte_mirror(row, 2),
            self.row_byte_mirror(row, 3),
            self.row_byte_mirror(row, 4),
            self.row_byte_mirror(row, 5),
        ]
    }

    pub const fn column_direct(&self, column: usize) -> GlyphDataRow {
        [
            self.column_byte_direct(column, 0),
            self.column_byte_direct(column, 1),
            self.column_byte_direct(column, 2),
            self.column_byte_direct(column, 3),
            self.column_byte_direct(column, 4),
            self.column_byte_direct(column, 5),
        ]
    }
    pub const fn column_mirror(&self, column: usize) -> GlyphDataRow {
        [
            self.column_byte_mirror(column, 0),
            self.column_byte_mirror(column, 1),
            self.column_byte_mirror(column, 2),
            self.column_byte_mirror(column, 3),
            self.column_byte_mirror(column, 4),
            self.column_byte_mirror(column, 5),
        ]
    }

    pub const fn glyph_up(&self) -> GlyphData {
        [
            self.row_mirror(0),
            self.row_mirror(1),
            self.row_mirror(2),
            self.row_mirror(3),
            self.row_mirror(4),
            self.row_mirror(5),
            self.row_mirror(6),
            self.row_mirror(7),
            self.row_mirror(8),
            self.row_mirror(9),
            self.row_mirror(10),
            self.row_mirror(11),
            self.row_mirror(12),
            self.row_mirror(13),
            self.row_mirror(14),
            self.row_mirror(15),
            self.row_mirror(16),
            self.row_mirror(17),
        ]
    }

    pub const fn glyph_down(&self) -> GlyphData {
        [
            self.row_direct(17),
            self.row_direct(16),
            self.row_direct(15),
            self.row_direct(14),
            self.row_direct(13),
            self.row_direct(12),
            self.row_direct(11),
            self.row_direct(10),
            self.row_direct(9),
            self.row_direct(8),
            self.row_direct(7),
            self.row_direct(6),
            self.row_direct(5),
            self.row_direct(4),
            self.row_direct(3),
            self.row_direct(2),
            self.row_direct(1),
            self.row_direct(0),
        ]
    }

    pub const fn glyph_left(&self) -> GlyphData {
        [
            self.column_direct(0),
            self.column_direct(1),
            self.column_direct(2),
            self.column_direct(3),
            self.column_direct(4),
            self.column_direct(5),
            self.column_direct(6),
            self.column_direct(7),
            self.column_direct(8),
            self.column_direct(9),
            self.column_direct(10),
            self.column_direct(11),
            self.column_direct(12),
            self.column_direct(13),
            self.column_direct(14),
            self.column_direct(15),
            self.column_direct(16),
            self.column_direct(17),
        ]
    }

    pub const fn glyph_right(&self) -> GlyphData {
        [
            self.column_mirror(17),
            self.column_mirror(16),
            self.column_mirror(15),
            self.column_mirror(14),
            self.column_mirror(13),
            self.column_mirror(12),
            self.column_mirror(11),
            self.column_mirror(10),
            self.column_mirror(9),
            self.column_mirror(8),
            self.column_mirror(7),
            self.column_mirror(6),
            self.column_mirror(5),
            self.column_mirror(4),
            self.column_mirror(3),
            self.column_mirror(2),
            self.column_mirror(1),
            self.column_mirror(0),
        ]
    }

    pub const fn glyph(&self, dir: LcdTopSide) -> GlyphData {
        match dir {
            LcdTopSide::Up => self.glyph_up(),
            LcdTopSide::Down => self.glyph_down(),
            LcdTopSide::Left => self.glyph_left(),
            LcdTopSide::Right => self.glyph_right(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LcdTopSide {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GlyphCode {
    Space,
    Full,
    D0,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    D9,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    UP,
    DOWN,
    LEFT,
    RIGHT,
    UpLEFT,
    UpRIGHT,
    TurnLEFT,
    TurnRIGHT,
    Plus,
    Minus,
    Yes,
    No,
    BarUp0,
    BarUp1,
    BarUp2,
    BarUp3,
    BarUp4,
    BarUp5,
    BarUp6,
    BarUp7,
    BarUp8,
    BarUp9,
    BarUp10,
    BarUp11,
    BarUp12,
    BarUp13,
    BarUp14,
    BarUp15,
    BarUp16,
    BarUp17,
    BarUp18,
    BarDown0,
    BarDown1,
    BarDown2,
    BarDown3,
    BarDown4,
    BarDown5,
    BarDown6,
    BarDown7,
    BarDown8,
    BarDown9,
    BarDown10,
    BarDown11,
    BarDown12,
    BarDown13,
    BarDown14,
    BarDown15,
    BarDown16,
    BarDown17,
    BarDown18,
    BarLeft0,
    BarLeft1,
    BarLeft2,
    BarLeft3,
    BarLeft4,
    BarLeft5,
    BarLeft6,
    BarLeft7,
    BarLeft8,
    BarLeft9,
    BarLeft10,
    BarLeft11,
    BarLeft12,
    BarLeft13,
    BarLeft14,
    BarLeft15,
    BarLeft16,
    BarLeft17,
    BarLeft18,
    BarRight0,
    BarRight1,
    BarRight2,
    BarRight3,
    BarRight4,
    BarRight5,
    BarRight6,
    BarRight7,
    BarRight8,
    BarRight9,
    BarRight10,
    BarRight11,
    BarRight12,
    BarRight13,
    BarRight14,
    BarRight15,
    BarRight16,
    BarRight17,
    BarRight18,
    Count,
}

const fn byte_to_glyph_code(byte: u8) -> GlyphCode {
    match byte {
        b' ' => GlyphCode::Space,
        b'_' => GlyphCode::Full,

        b'0' => GlyphCode::D0,
        b'1' => GlyphCode::D1,
        b'2' => GlyphCode::D2,
        b'3' => GlyphCode::D3,
        b'4' => GlyphCode::D4,
        b'5' => GlyphCode::D5,
        b'6' => GlyphCode::D6,
        b'7' => GlyphCode::D7,
        b'8' => GlyphCode::D8,
        b'9' => GlyphCode::D9,

        b'A' => GlyphCode::A,
        b'B' => GlyphCode::B,
        b'C' => GlyphCode::C,
        b'D' => GlyphCode::D,
        b'E' => GlyphCode::E,
        b'F' => GlyphCode::F,
        b'G' => GlyphCode::G,
        b'H' => GlyphCode::H,
        b'I' => GlyphCode::I,
        b'J' => GlyphCode::J,
        b'K' => GlyphCode::K,
        b'L' => GlyphCode::L,
        b'M' => GlyphCode::M,
        b'N' => GlyphCode::N,
        b'O' => GlyphCode::O,
        b'P' => GlyphCode::P,
        b'Q' => GlyphCode::Q,
        b'R' => GlyphCode::R,
        b'S' => GlyphCode::S,
        b'T' => GlyphCode::T,
        b'U' => GlyphCode::U,
        b'V' => GlyphCode::V,
        b'W' => GlyphCode::W,
        b'X' => GlyphCode::X,
        b'Y' => GlyphCode::Y,
        b'Z' => GlyphCode::Z,

        b'^' => GlyphCode::UP,
        b'v' => GlyphCode::DOWN,
        b'<' => GlyphCode::LEFT,
        b'>' => GlyphCode::RIGHT,
        b'\\' => GlyphCode::UpLEFT,
        b'/' => GlyphCode::UpRIGHT,
        b'l' => GlyphCode::TurnLEFT,
        b'r' => GlyphCode::TurnRIGHT,
        b'+' => GlyphCode::Plus,
        b'-' => GlyphCode::Minus,
        b'y' => GlyphCode::Yes,
        b'n' => GlyphCode::No,

        _ => GlyphCode::Full,
    }
}

const fn glyph_digit(digit: i32) -> GlyphCode {
    match digit {
        0 => GlyphCode::D0,
        1 => GlyphCode::D1,
        2 => GlyphCode::D2,
        3 => GlyphCode::D3,
        4 => GlyphCode::D4,
        5 => GlyphCode::D5,
        6 => GlyphCode::D6,
        7 => GlyphCode::D7,
        8 => GlyphCode::D8,
        9 => GlyphCode::D9,
        _ => GlyphCode::Full,
    }
}

impl GlyphCode {
    pub fn from(index: usize) -> Self {
        if index < GlyphCode::Count as usize {
            unsafe { core::mem::transmute(index as u8) }
        } else {
            GlyphCode::Space
        }
    }
}

pub struct Font<TOP: LcdTopSideProvider> {
    top: PhantomData<TOP>,
    glyphs: [GlyphData; GlyphCode::Count as usize],
}

pub const fn build_font<TOP: LcdTopSideProvider>() -> Font<TOP> {
    let dir = TOP::TOP_SIDE;
    Font {
        top: PhantomData,
        glyphs: [
            GLYPH_SPACE.glyph(dir),
            GLYPH_FULL.glyph(dir),
            GLYPH_0.glyph(dir),
            GLYPH_1.glyph(dir),
            GLYPH_2.glyph(dir),
            GLYPH_3.glyph(dir),
            GLYPH_4.glyph(dir),
            GLYPH_5.glyph(dir),
            GLYPH_6.glyph(dir),
            GLYPH_7.glyph(dir),
            GLYPH_8.glyph(dir),
            GLYPH_9.glyph(dir),
            GLYPH_A.glyph(dir),
            GLYPH_B.glyph(dir),
            GLYPH_C.glyph(dir),
            GLYPH_D.glyph(dir),
            GLYPH_E.glyph(dir),
            GLYPH_F.glyph(dir),
            GLYPH_G.glyph(dir),
            GLYPH_H.glyph(dir),
            GLYPH_I.glyph(dir),
            GLYPH_J.glyph(dir),
            GLYPH_K.glyph(dir),
            GLYPH_L.glyph(dir),
            GLYPH_M.glyph(dir),
            GLYPH_N.glyph(dir),
            GLYPH_O.glyph(dir),
            GLYPH_P.glyph(dir),
            GLYPH_Q.glyph(dir),
            GLYPH_R.glyph(dir),
            GLYPH_S.glyph(dir),
            GLYPH_T.glyph(dir),
            GLYPH_U.glyph(dir),
            GLYPH_V.glyph(dir),
            GLYPH_W.glyph(dir),
            GLYPH_X.glyph(dir),
            GLYPH_Y.glyph(dir),
            GLYPH_Z.glyph(dir),
            GLYPH_UP.glyph(dir),
            GLYPH_DOWN.glyph(dir),
            GLYPH_LEFT.glyph(dir),
            GLYPH_RIGHT.glyph(dir),
            GLYPH_UP_LEFT.glyph(dir),
            GLYPH_UP_RIGHT.glyph(dir),
            GLYPH_TURN_LEFT.glyph(dir),
            GLYPH_TURN_RIGHT.glyph(dir),
            GLYPH_PLUS.glyph(dir),
            GLYPH_MINUS.glyph(dir),
            GLYPH_YES.glyph(dir),
            GLYPH_NO.glyph(dir),
            glyph_bar_up(0).glyph(dir),
            glyph_bar_up(1).glyph(dir),
            glyph_bar_up(2).glyph(dir),
            glyph_bar_up(3).glyph(dir),
            glyph_bar_up(4).glyph(dir),
            glyph_bar_up(5).glyph(dir),
            glyph_bar_up(6).glyph(dir),
            glyph_bar_up(7).glyph(dir),
            glyph_bar_up(8).glyph(dir),
            glyph_bar_up(9).glyph(dir),
            glyph_bar_up(10).glyph(dir),
            glyph_bar_up(11).glyph(dir),
            glyph_bar_up(12).glyph(dir),
            glyph_bar_up(13).glyph(dir),
            glyph_bar_up(14).glyph(dir),
            glyph_bar_up(15).glyph(dir),
            glyph_bar_up(16).glyph(dir),
            glyph_bar_up(17).glyph(dir),
            glyph_bar_up(18).glyph(dir),
            glyph_bar_down(0).glyph(dir),
            glyph_bar_down(1).glyph(dir),
            glyph_bar_down(2).glyph(dir),
            glyph_bar_down(3).glyph(dir),
            glyph_bar_down(4).glyph(dir),
            glyph_bar_down(5).glyph(dir),
            glyph_bar_down(6).glyph(dir),
            glyph_bar_down(7).glyph(dir),
            glyph_bar_down(8).glyph(dir),
            glyph_bar_down(9).glyph(dir),
            glyph_bar_down(10).glyph(dir),
            glyph_bar_down(11).glyph(dir),
            glyph_bar_down(12).glyph(dir),
            glyph_bar_down(13).glyph(dir),
            glyph_bar_down(14).glyph(dir),
            glyph_bar_down(15).glyph(dir),
            glyph_bar_down(16).glyph(dir),
            glyph_bar_down(17).glyph(dir),
            glyph_bar_down(18).glyph(dir),
            glyph_bar_left(0).glyph(dir),
            glyph_bar_left(1).glyph(dir),
            glyph_bar_left(2).glyph(dir),
            glyph_bar_left(3).glyph(dir),
            glyph_bar_left(4).glyph(dir),
            glyph_bar_left(5).glyph(dir),
            glyph_bar_left(6).glyph(dir),
            glyph_bar_left(7).glyph(dir),
            glyph_bar_left(8).glyph(dir),
            glyph_bar_left(9).glyph(dir),
            glyph_bar_left(10).glyph(dir),
            glyph_bar_left(11).glyph(dir),
            glyph_bar_left(12).glyph(dir),
            glyph_bar_left(13).glyph(dir),
            glyph_bar_left(14).glyph(dir),
            glyph_bar_left(15).glyph(dir),
            glyph_bar_left(16).glyph(dir),
            glyph_bar_left(17).glyph(dir),
            glyph_bar_left(18).glyph(dir),
            glyph_bar_right(0).glyph(dir),
            glyph_bar_right(1).glyph(dir),
            glyph_bar_right(2).glyph(dir),
            glyph_bar_right(3).glyph(dir),
            glyph_bar_right(4).glyph(dir),
            glyph_bar_right(5).glyph(dir),
            glyph_bar_right(6).glyph(dir),
            glyph_bar_right(7).glyph(dir),
            glyph_bar_right(8).glyph(dir),
            glyph_bar_right(9).glyph(dir),
            glyph_bar_right(10).glyph(dir),
            glyph_bar_right(11).glyph(dir),
            glyph_bar_right(12).glyph(dir),
            glyph_bar_right(13).glyph(dir),
            glyph_bar_right(14).glyph(dir),
            glyph_bar_right(15).glyph(dir),
            glyph_bar_right(16).glyph(dir),
            glyph_bar_right(17).glyph(dir),
            glyph_bar_right(18).glyph(dir),
        ],
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WindowId {
    Main,
    Sub1,
    Sub2,
}

impl WindowId {
    pub fn size<TOP: LcdTopSideProvider>(&self) -> usize {
        match self {
            WindowId::Main => TOP::MAIN_WINDOW_SIZE,
            WindowId::Sub1 => TOP::SUB1_WINDOW_SIZE,
            WindowId::Sub2 => TOP::SUB2_WINDOW_SIZE,
        }
    }
    pub fn x<TOP: LcdTopSideProvider>(&self) -> usize {
        match self {
            WindowId::Main => TOP::MAIN_WINDOW_X,
            WindowId::Sub1 => TOP::SUB1_WINDOW_X,
            WindowId::Sub2 => TOP::SUB2_WINDOW_X,
        }
    }
    pub fn y<TOP: LcdTopSideProvider>(&self) -> usize {
        match self {
            WindowId::Main => TOP::MAIN_WINDOW_Y,
            WindowId::Sub1 => TOP::SUB1_WINDOW_Y,
            WindowId::Sub2 => TOP::SUB2_WINDOW_Y,
        }
    }
}

pub trait WindowIdProvider {
    const ID: WindowId;
    fn kind() -> WindowId {
        Self::ID
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct WindowIdMain {}
impl WindowIdProvider for WindowIdMain {
    const ID: WindowId = WindowId::Main;
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct WindowIdSub1 {}
impl WindowIdProvider for WindowIdSub1 {
    const ID: WindowId = WindowId::Sub1;
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct WindowIdSub2 {}
impl WindowIdProvider for WindowIdSub2 {
    const ID: WindowId = WindowId::Sub2;
}

pub trait LcdTopSideProvider {
    const TOP_SIDE: LcdTopSide;
    const MAIN_WINDOW_SIZE: usize;
    const MAIN_WINDOW_X: usize;
    const MAIN_WINDOW_Y: usize;
    const SUB1_WINDOW_SIZE: usize;
    const SUB1_WINDOW_X: usize;
    const SUB1_WINDOW_Y: usize;
    const SUB2_WINDOW_SIZE: usize;
    const SUB2_WINDOW_X: usize;
    const SUB2_WINDOW_Y: usize;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LcdTopSideUp;
impl LcdTopSideProvider for LcdTopSideUp {
    const TOP_SIDE: LcdTopSide = LcdTopSide::Up;
    const MAIN_WINDOW_SIZE: usize = 7;
    const MAIN_WINDOW_X: usize = 0;
    const MAIN_WINDOW_Y: usize = 0;
    const SUB1_WINDOW_SIZE: usize = 3;
    const SUB1_WINDOW_X: usize = 7;
    const SUB1_WINDOW_Y: usize = 0;
    const SUB2_WINDOW_SIZE: usize = 3;
    const SUB2_WINDOW_X: usize = 7;
    const SUB2_WINDOW_Y: usize = 4;
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LcdTopSideDown;
impl LcdTopSideProvider for LcdTopSideDown {
    const TOP_SIDE: LcdTopSide = LcdTopSide::Down;
    const MAIN_WINDOW_SIZE: usize = 7;
    const MAIN_WINDOW_X: usize = 3;
    const MAIN_WINDOW_Y: usize = 0;
    const SUB1_WINDOW_SIZE: usize = 3;
    const SUB1_WINDOW_X: usize = 0;
    const SUB1_WINDOW_Y: usize = 4;
    const SUB2_WINDOW_SIZE: usize = 3;
    const SUB2_WINDOW_X: usize = 0;
    const SUB2_WINDOW_Y: usize = 0;
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LcdTopSideLeft;
impl LcdTopSideProvider for LcdTopSideLeft {
    const TOP_SIDE: LcdTopSide = LcdTopSide::Left;
    const MAIN_WINDOW_SIZE: usize = 7;
    const MAIN_WINDOW_X: usize = 0;
    const MAIN_WINDOW_Y: usize = 0;
    const SUB1_WINDOW_SIZE: usize = 3;
    const SUB1_WINDOW_X: usize = 7;
    const SUB1_WINDOW_Y: usize = 4;
    const SUB2_WINDOW_SIZE: usize = 3;
    const SUB2_WINDOW_X: usize = 7;
    const SUB2_WINDOW_Y: usize = 0;
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LcdTopSideRight;
impl LcdTopSideProvider for LcdTopSideRight {
    const TOP_SIDE: LcdTopSide = LcdTopSide::Right;
    const MAIN_WINDOW_SIZE: usize = 7;
    const MAIN_WINDOW_X: usize = 3;
    const MAIN_WINDOW_Y: usize = 0;
    const SUB1_WINDOW_SIZE: usize = 3;
    const SUB1_WINDOW_X: usize = 0;
    const SUB1_WINDOW_Y: usize = 0;
    const SUB2_WINDOW_SIZE: usize = 3;
    const SUB2_WINDOW_X: usize = 0;
    const SUB2_WINDOW_Y: usize = 4;
}

pub struct LcdCell {
    cell: Cell<GlyphCode>,
}

const LCD_CELL_EMPTY: LcdCell = LcdCell {
    cell: Cell::new(GlyphCode::Space),
};
const LCD_CELL_FULL: LcdCell = LcdCell {
    cell: Cell::new(GlyphCode::Full),
};

impl LcdCell {
    pub fn get(&self) -> GlyphCode {
        self.cell.get()
    }

    pub fn set(&self, c: GlyphCode) {
        self.cell.set(c);
    }
}
unsafe impl Sync for LcdCell {}

pub struct LcdScreenNext {
    cells: alloc::boxed::Box<[LcdCell; LCD_GLYPHS_PER_ROW * LCD_GLYPHS_PER_COLUMN]>,
}

impl LcdScreenNext {
    pub fn new() -> Self {
        Self {
            cells: alloc::boxed::Box::new(
                [LCD_CELL_EMPTY; LCD_GLYPHS_PER_ROW * LCD_GLYPHS_PER_COLUMN],
            ),
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        ((y * LCD_GLYPHS_PER_ROW) + x) % self.cells.len()
    }

    pub fn get(&self, x: usize, y: usize) -> GlyphCode {
        self.cells[self.index(x, y)].get()
    }

    pub fn set(&self, x: usize, y: usize, c: GlyphCode) {
        self.cells[self.index(x, y)].set(c);
    }

    pub fn clear(&self) {
        for cell in self.cells.iter() {
            cell.set(GlyphCode::Space);
        }
    }
}

pub struct LcdScreenCurrent<TOP: LcdTopSideProvider + 'static> {
    font: &'static Font<TOP>,
    cells: alloc::boxed::Box<[LcdCell; LCD_GLYPHS_PER_ROW * LCD_GLYPHS_PER_COLUMN]>,
}

impl<TOP: LcdTopSideProvider + 'static> LcdScreenCurrent<TOP> {
    pub fn new(font: &'static Font<TOP>) -> Self {
        Self {
            font,
            cells: alloc::boxed::Box::new(
                [LCD_CELL_FULL; LCD_GLYPHS_PER_ROW * LCD_GLYPHS_PER_COLUMN],
            ),
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        ((y * LCD_GLYPHS_PER_ROW) + x) % self.cells.len()
    }

    pub fn get(&self, x: usize, y: usize) -> GlyphCode {
        self.cells[self.index(x, y)].get()
    }

    pub fn set(&self, x: usize, y: usize, c: GlyphCode) {
        self.cells[self.index(x, y)].set(c);
    }

    pub fn draw_glyph(&self, c: GlyphCode, x: usize, y: usize) {
        lcd_apply(|fb| {
            const STRIDE: usize = LCD_FRAMEBUFFER_ROW_BYTES;
            let mut base = (y * STRIDE * GLYPH_SIZE) + (x * GLYPH_DATA_ROW_BYTES);
            for glypg_row_src in self.font.glyphs[c as usize].iter() {
                let glyph_row = &mut fb[base..base + GLYPH_DATA_ROW_BYTES];
                glyph_row.copy_from_slice(glypg_row_src);
                base += STRIDE;
            }
        });
    }

    pub fn apply(&self, x: usize, y: usize, next: GlyphCode) {
        let current = self.get(x, y);
        if current != next {
            self.set(x, y, next);
            self.draw_glyph(next, x, y);
        }
    }

    pub fn refresh(&self, next: &LcdScreenNext) {
        for y in 0..LCD_GLYPHS_PER_COLUMN {
            for x in 0..LCD_GLYPHS_PER_ROW {
                let c = next.get(x, y);
                self.apply(x, y, c);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct BarParameters {
    full_blocks: usize,
    empty_blocks: usize,
    remainder: usize,
}

impl BarParameters {
    pub fn new(value: i32, limit: i32, size: usize) -> Self {
        let max = size * GLYPH_SIZE;
        let value = (value as usize) * max / (limit as usize);
        let full_blocks = value / GLYPH_SIZE;
        let empty_blocks = if (full_blocks + 1) < size {
            size - (full_blocks + 1)
        } else {
            0
        };
        let remainder = value % GLYPH_SIZE;
        Self {
            full_blocks,
            empty_blocks,
            remainder,
        }
    }
}

struct Window<ID: WindowIdProvider, TOP: LcdTopSideProvider> {
    id: PhantomData<ID>,
    top: PhantomData<TOP>,
}

impl<ID: WindowIdProvider, TOP: LcdTopSideProvider> Window<ID, TOP> {
    pub const fn new() -> Self {
        Self {
            id: PhantomData,
            top: PhantomData,
        }
    }

    pub fn transform(&self, x: usize, y: usize) -> (usize, usize) {
        let size = ID::kind().size::<TOP>();
        let (lx, ly) = match TOP::TOP_SIDE {
            LcdTopSide::Up => (x, y),
            LcdTopSide::Down => (size - 1 - x, size - 1 - y),
            LcdTopSide::Left => (y, size - 1 - x),
            LcdTopSide::Right => (size - 1 - y, x),
        };
        (ID::kind().x::<TOP>() + lx, ID::kind().y::<TOP>() + ly)
    }

    pub fn set(&self, x: usize, y: usize, c: GlyphCode, screen: &LcdScreenNext) {
        let (x, y) = self.transform(x, y);
        screen.set(x, y, c);
    }

    pub fn print_char(&self, x: usize, y: usize, c: char, screen: &LcdScreenNext) {
        let (x, y) = self.transform(x, y);
        let c = byte_to_glyph_code(c as u8);
        screen.set(x, y, c);
    }

    pub fn print_row(&self, row: usize, text: &str, screen: &LcdScreenNext) {
        let size = ID::kind().size::<TOP>();
        let mut x = 0;
        let mut y = row;
        for byte in text.bytes() {
            let c = byte_to_glyph_code(byte);
            self.set(x, y, c, screen);
            x += 1;
            if x >= size {
                x = 0;
                y += 1;
                if y >= size {
                    break;
                }
            }
        }
    }

    pub fn print_column(&self, column: usize, text: &str, screen: &LcdScreenNext) {
        let size = ID::kind().size::<TOP>();
        let mut x = column;
        let mut y = 0;
        for byte in text.bytes() {
            let c = byte_to_glyph_code(byte);
            self.set(x, y, c, screen);
            y += 1;
            if y >= size {
                y = 0;
                x += 1;
                if x >= size {
                    break;
                }
            }
        }
    }

    pub fn print_value(
        &self,
        row: usize,
        column: usize,
        digits: usize,
        value: i32,
        screen: &LcdScreenNext,
    ) {
        let mut x = column + digits - 1;
        let y = row;
        let mut value = value;
        let mut is_first = true;
        for _ in 0..digits {
            let digit = value % 10;
            let c = if value == 0 {
                if is_first {
                    GlyphCode::D0
                } else {
                    GlyphCode::Space
                }
            } else {
                glyph_digit(digit)
            };
            self.set(x, y, c, screen);
            x -= 1;
            value /= 10;
            is_first = false;
        }
    }

    pub fn print_value_signed(
        &self,
        row: usize,
        column: usize,
        digits: usize,
        value: i32,
        screen: &LcdScreenNext,
    ) {
        let (c, abs_value) = if value > 0 {
            (GlyphCode::Plus, value)
        } else if value < 0 {
            (GlyphCode::Minus, -value)
        } else {
            (GlyphCode::Space, 0)
        };
        self.set(column, row, c, screen);
        self.print_value(row, column + 1, digits, abs_value, screen);
    }

    pub fn bar_horizontal(&self, row: usize, value: i32, limit: i32, screen: &LcdScreenNext) {
        let value = value.min(limit).max(-limit);
        let size = ID::kind().size::<TOP>();
        if value > 0 {
            let params = BarParameters::new(value, limit, size);
            let mut x = 0;
            for _ in 0..params.full_blocks {
                self.set(x, row, GlyphCode::BarLeft18, screen);
                x += 1;
            }
            if x < size {
                let c = GlyphCode::from(GlyphCode::BarLeft0 as usize + params.remainder);
                self.set(x, row, c, screen);
                x += 1;
            }
            if x < size {
                for _ in 0..params.empty_blocks {
                    self.set(x, row, GlyphCode::BarLeft0, screen);
                    x += 1;
                }
            }
        } else if value < 0 {
            let params = BarParameters::new(-value, limit, size);
            let mut x = (size - 1) as isize;
            for _ in 0..params.full_blocks {
                self.set(x as usize, row, GlyphCode::BarRight18, screen);
                x -= 1;
            }
            if x >= 0 {
                let c = GlyphCode::from(GlyphCode::BarRight0 as usize + params.remainder);
                self.set(x as usize, row, c, screen);
                x -= 1;
            }
            if x >= 0 {
                for _ in 0..params.empty_blocks {
                    self.set(x as usize, row, GlyphCode::Space, screen);
                    x -= 1;
                }
            }
        } else {
            for x in 0..size {
                self.set(x, row, GlyphCode::BarRight0, screen);
            }
        }
    }

    pub fn bar_vertical(&self, column: usize, value: i32, limit: i32, screen: &LcdScreenNext) {
        let value = value.min(limit).max(-limit);
        let size = ID::kind().size::<TOP>();
        if value > 0 {
            let params = BarParameters::new(value, limit, size);
            let mut y = (size - 1) as isize;
            for _ in 0..params.full_blocks {
                self.set(column, y as usize, GlyphCode::BarUp18, screen);
                y -= 1;
            }
            if y >= 0 {
                let c = GlyphCode::from(GlyphCode::BarUp0 as usize + params.remainder);
                self.set(column, y as usize, c, screen);
                y -= 1;
            }
            if y >= 0 {
                for _ in 0..params.empty_blocks {
                    self.set(column, y as usize, GlyphCode::BarUp0, screen);
                    y -= 1;
                }
            }
        } else if value < 0 {
            let params = BarParameters::new(-value, limit, size);
            let mut y = 0;
            for _ in 0..params.full_blocks {
                self.set(column, y, GlyphCode::BarDown18, screen);
                y += 1;
            }
            if y < size {
                let c = GlyphCode::from(GlyphCode::BarDown0 as usize + params.remainder);
                self.set(column, y, c, screen);
                y += 1;
            }
            if y < size {
                for _ in 0..params.empty_blocks {
                    self.set(column, y, GlyphCode::BarDown0, screen);
                    y += 1;
                }
            }
        } else {
            for y in 0..size {
                self.set(column, y, GlyphCode::Space, screen);
            }
        }
    }
}

pub trait LcdTrait {
    fn clear(&self);

    fn print_char(&self, x: usize, y: usize, c: char);
    fn print(&self, row: usize, text: &str);
    fn print_column(&self, column: usize, text: &str);
    fn print_value(&self, row: usize, column: usize, digits: usize, value: i32);
    fn print_value_signed(&self, row: usize, column: usize, digits: usize, value: i32);
    fn bar_vertical(&self, column: usize, value: i32, limit: i32);
    fn bar_horizontal(&self, row: usize, value: i32, limit: i32);

    fn print_char_sub1(&self, x: usize, y: usize, c: char);
    fn print_sub1(&self, row: usize, text: &str);
    fn print_column_sub1(&self, column: usize, text: &str);
    fn print_value_sub1(&self, row: usize, column: usize, digits: usize, value: i32);
    fn print_value_signed_sub1(&self, row: usize, column: usize, digits: usize, value: i32);
    fn bar_vertical_sub1(&self, column: usize, value: i32, limit: i32);
    fn bar_horizontal_sub1(&self, row: usize, value: i32, limit: i32);

    fn print_char_sub2(&self, x: usize, y: usize, c: char);
    fn print_sub2(&self, row: usize, text: &str);
    fn print_column_sub2(&self, column: usize, text: &str);
    fn print_value_sub2(&self, row: usize, column: usize, digits: usize, value: i32);
    fn print_value_signed_sub2(&self, row: usize, column: usize, digits: usize, value: i32);
    fn bar_vertical_sub2(&self, column: usize, value: i32, limit: i32);
    fn bar_horizontal_sub2(&self, row: usize, value: i32, limit: i32);
}

impl<TOP: LcdTopSideProvider + 'static> LcdTrait for Lcd<TOP> {
    fn clear(&self) {
        self.clear();
    }

    fn print_char(&self, x: usize, y: usize, c: char) {
        self.print_char(x, y, c);
    }

    fn print(&self, row: usize, text: &str) {
        self.print(row, text);
    }

    fn print_column(&self, column: usize, text: &str) {
        self.print_column(column, text);
    }

    fn print_value(&self, row: usize, column: usize, digits: usize, value: i32) {
        self.print_value(row, column, digits, value);
    }

    fn print_value_signed(&self, row: usize, column: usize, digits: usize, value: i32) {
        self.print_value_signed(row, column, digits, value);
    }

    fn bar_vertical(&self, column: usize, value: i32, limit: i32) {
        self.bar_vertical(column, value, limit);
    }

    fn bar_horizontal(&self, row: usize, value: i32, limit: i32) {
        self.bar_horizontal(row, value, limit);
    }

    fn print_char_sub1(&self, x: usize, y: usize, c: char) {
        self.print_char_sub1(x, y, c);
    }

    fn print_sub1(&self, row: usize, text: &str) {
        self.print_sub1(row, text);
    }

    fn print_column_sub1(&self, column: usize, text: &str) {
        self.print_column_sub1(column, text);
    }

    fn print_value_sub1(&self, row: usize, column: usize, digits: usize, value: i32) {
        self.print_value_sub1(row, column, digits, value);
    }

    fn print_value_signed_sub1(&self, row: usize, column: usize, digits: usize, value: i32) {
        self.print_value_signed_sub1(row, column, digits, value);
    }

    fn bar_vertical_sub1(&self, column: usize, value: i32, limit: i32) {
        self.bar_vertical_sub1(column, value, limit);
    }

    fn bar_horizontal_sub1(&self, row: usize, value: i32, limit: i32) {
        self.bar_horizontal_sub1(row, value, limit);
    }

    fn print_char_sub2(&self, x: usize, y: usize, c: char) {
        self.print_char_sub2(x, y, c);
    }

    fn print_sub2(&self, row: usize, text: &str) {
        self.print_sub2(row, text);
    }

    fn print_column_sub2(&self, column: usize, text: &str) {
        self.print_column_sub2(column, text);
    }

    fn print_value_sub2(&self, row: usize, column: usize, digits: usize, value: i32) {
        self.print_value_sub2(row, column, digits, value);
    }

    fn print_value_signed_sub2(&self, row: usize, column: usize, digits: usize, value: i32) {
        self.print_value_signed_sub2(row, column, digits, value);
    }

    fn bar_vertical_sub2(&self, column: usize, value: i32, limit: i32) {
        self.bar_vertical_sub2(column, value, limit);
    }

    fn bar_horizontal_sub2(&self, row: usize, value: i32, limit: i32) {
        self.bar_horizontal_sub2(row, value, limit);
    }
}

pub struct Lcd<TOP: LcdTopSideProvider + 'static> {
    top: PhantomData<TOP>,
    pub current_screen: LcdScreenCurrent<TOP>,
    pub next_screen: LcdScreenNext,
    main_window: Window<WindowIdMain, TOP>,
    sub1_window: Window<WindowIdSub1, TOP>,
    sub2_window: Window<WindowIdSub2, TOP>,
}

impl<TOP: LcdTopSideProvider + 'static> Lcd<TOP> {
    pub fn new(font: &'static Font<TOP>) -> Self {
        Self {
            top: PhantomData,
            current_screen: LcdScreenCurrent::new(font),
            next_screen: LcdScreenNext::new(),
            main_window: Window::new(),
            sub1_window: Window::new(),
            sub2_window: Window::new(),
        }
    }

    pub fn clear_pixels(&self) {
        lcd_apply(|fb| {
            fb.fill(0);
        });
    }

    pub fn clear(&self) {
        self.next_screen.clear();
    }

    pub fn refresh(&self) {
        self.current_screen.refresh(&self.next_screen);
    }

    pub fn draw_glyph(&self, c: GlyphCode, x: usize, y: usize) {
        self.current_screen.draw_glyph(c, x, y);
    }

    pub fn print_char(&self, x: usize, y: usize, c: char) {
        self.main_window.print_char(x, y, c, &self.next_screen);
    }
    pub fn print(&self, row: usize, text: &str) {
        self.main_window.print_row(row, text, &self.next_screen);
    }
    pub fn print_column(&self, column: usize, text: &str) {
        self.main_window
            .print_column(column, text, &self.next_screen);
    }
    pub fn print_value(&self, row: usize, column: usize, digits: usize, value: i32) {
        self.main_window
            .print_value(row, column, digits, value, &self.next_screen);
    }
    pub fn print_value_signed(&self, row: usize, column: usize, digits: usize, value: i32) {
        self.main_window
            .print_value_signed(row, column, digits, value, &self.next_screen);
    }
    pub fn bar_vertical(&self, column: usize, value: i32, limit: i32) {
        self.main_window
            .bar_vertical(column, value, limit, &self.next_screen);
    }
    pub fn bar_horizontal(&self, row: usize, value: i32, limit: i32) {
        self.main_window
            .bar_horizontal(row, value, limit, &self.next_screen);
    }

    pub fn print_char_sub1(&self, x: usize, y: usize, c: char) {
        self.sub1_window.print_char(x, y, c, &self.next_screen);
    }
    pub fn print_sub1(&self, row: usize, text: &str) {
        self.sub1_window.print_row(row, text, &self.next_screen);
    }
    pub fn print_column_sub1(&self, column: usize, text: &str) {
        self.sub1_window
            .print_column(column, text, &self.next_screen);
    }
    pub fn print_value_sub1(&self, row: usize, column: usize, digits: usize, value: i32) {
        self.sub1_window
            .print_value(row, column, digits, value, &self.next_screen);
    }
    pub fn print_value_signed_sub1(&self, row: usize, column: usize, digits: usize, value: i32) {
        self.sub1_window
            .print_value_signed(row, column, digits, value, &self.next_screen);
    }
    pub fn bar_horizontal_sub1(&self, row: usize, value: i32, limit: i32) {
        self.sub1_window
            .bar_horizontal(row, value, limit, &self.next_screen);
    }
    pub fn bar_vertical_sub1(&self, column: usize, value: i32, limit: i32) {
        self.sub1_window
            .bar_vertical(column, value, limit, &self.next_screen);
    }

    pub fn print_char_sub2(&self, x: usize, y: usize, c: char) {
        self.sub2_window.print_char(x, y, c, &self.next_screen);
    }
    pub fn print_sub2(&self, row: usize, text: &str) {
        self.sub2_window.print_row(row, text, &self.next_screen);
    }
    pub fn print_column_sub2(&self, column: usize, text: &str) {
        self.sub2_window
            .print_column(column, text, &self.next_screen);
    }
    pub fn print_value_sub2(&self, row: usize, column: usize, digits: usize, value: i32) {
        self.sub2_window
            .print_value(row, column, digits, value, &self.next_screen);
    }
    pub fn print_value_signed_sub2(&self, row: usize, column: usize, digits: usize, value: i32) {
        self.sub2_window
            .print_value_signed(row, column, digits, value, &self.next_screen);
    }
    pub fn bar_horizontal_sub2(&self, row: usize, value: i32, limit: i32) {
        self.sub2_window
            .bar_horizontal(row, value, limit, &self.next_screen);
    }
    pub fn bar_vertical_sub2(&self, column: usize, value: i32, limit: i32) {
        self.sub2_window
            .bar_vertical(column, value, limit, &self.next_screen);
    }
}

const GLYPH_SPACE: GlyphSource = GlyphSource::new([0; GLYPH_SIZE]);
const GLYPH_FULL: GlyphSource = GlyphSource::new([0b111111111111111111; GLYPH_SIZE]);

const GLYPH_0: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b001110000000011100,
    0b001110000000011100,
    0b001110000000011100,
    0b001110000000011100,
    0b001110000000011100,
    0b001110000000011100,
    0b001110000000011100,
    0b001110000000011100,
    0b001110000000011100,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b000000000000000000,
]);

const GLYPH_1: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000000000,
]);

const GLYPH_2: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b001110000000000000,
    0b001110000000000000,
    0b001110000000000000,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b000000000000000000,
]);

const GLYPH_3: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b000000000000000000,
]);

const GLYPH_4: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b001110000000011100,
    0b001110000000011100,
    0b001110000000011100,
    0b001110000000011100,
    0b001110000000011100,
    0b001110000000011100,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000000000,
]);

const GLYPH_5: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b001110000000000000,
    0b001110000000000000,
    0b001110000000000000,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b000000000000000000,
]);

const GLYPH_6: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b001110000000000000,
    0b001110000000000000,
    0b001110000000000000,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b001110000000011100,
    0b001110000000011100,
    0b001110000000011100,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b000000000000000000,
]);

const GLYPH_7: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000000000,
]);

const GLYPH_8: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b001110000000011100,
    0b001110000000011100,
    0b001110000000011100,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b001110000000011100,
    0b001110000000011100,
    0b001110000000011100,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b000000000000000000,
]);

const GLYPH_9: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b001110000000011100,
    0b001110000000011100,
    0b001110000000011100,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b000000000000011100,
    0b000000000000011100,
    0b000000000000011100,
    0b001111111111111100,
    0b001111111111111100,
    0b001111111111111100,
    0b000000000000000000,
]);

const GLYPH_PLUS: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b000000000000000000,
    0b000000001110000000,
    0b000000001110000000,
    0b000000001110000000,
    0b000000001110000000,
    0b000000001110000000,
    0b000111111111111100,
    0b000111111111111100,
    0b000111111111111100,
    0b000000001110000000,
    0b000000001110000000,
    0b000000001110000000,
    0b000000001110000000,
    0b000000001110000000,
    0b000000000000000000,
    0b000000000000000000,
]);
const GLYPH_MINUS: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b000000000000000000,
    0b000000000000000000,
    0b000000000000000000,
    0b000000000000000000,
    0b000000000000000000,
    0b000000000000000000,
    0b000111111111111100,
    0b000111111111111100,
    0b000111111111111100,
    0b000000000000000000,
    0b000000000000000000,
    0b000000000000000000,
    0b000000000000000000,
    0b000000000000000000,
    0b000000000000000000,
    0b000000000000000000,
]);

const GLYPH_A: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b000000011110000000,
    0b000001111111100000,
    0b000011111111110000,
    0b000111000000111000,
    0b001110000000011100,
    0b001110000000011100,
    0b011111111111111110,
    0b011111111111111110,
    0b011111111111111110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b000000000000000000,
]);
const GLYPH_B: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b011111111111100000,
    0b011111111111111000,
    0b011111111111111100,
    0b011100000000011100,
    0b011100000000001110,
    0b011100000000011110,
    0b011111111111111100,
    0b011111111111111000,
    0b011111111111111100,
    0b011100000000011110,
    0b011100000000001110,
    0b011100000000011100,
    0b011111111111111100,
    0b011111111111111000,
    0b01111111111100000,
    0b000000000000000000,
]);
const GLYPH_C: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b000001111111100000,
    0b000111111111111000,
    0b001111111111111100,
    0b001110000000011100,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b001110000000011100,
    0b001111111111111100,
    0b000111111111111000,
    0b000001111111100000,
    0b000000000000000000,
]);
const GLYPH_D: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b011111111111100000,
    0b011111111111111000,
    0b011111111111111100,
    0b011100000000011100,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000011100,
    0b011111111111111100,
    0b011111111111111000,
    0b011111111111100000,
    0b000000000000000000,
]);
const GLYPH_E: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b011111111111111110,
    0b011111111111111110,
    0b011111111111111110,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011111111111111110,
    0b011111111111111110,
    0b011111111111111110,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011111111111111110,
    0b011111111111111110,
    0b011111111111111110,
    0b000000000000000000,
]);
const GLYPH_F: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b011111111111111110,
    0b011111111111111110,
    0b011111111111111110,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011111111111111110,
    0b011111111111111110,
    0b011111111111111110,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b000000000000000000,
]);
const GLYPH_G: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b000001111111100000,
    0b000111111111111000,
    0b001111111111111100,
    0b001110000000011100,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000011111100,
    0b011100000011111100,
    0b011100000011111100,
    0b001110000000011100,
    0b001111111111111100,
    0b000111111111111000,
    0b000001111111100000,
    0b000000000000000000,
]);
const GLYPH_H: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011111111111111110,
    0b011111111111111110,
    0b011111111111111110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b000000000000000000,
]);
const GLYPH_I: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000000000000000,
]);
const GLYPH_J: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b000000000000001110,
    0b000000000000001110,
    0b000000000000001110,
    0b000000000000001110,
    0b000000000000001110,
    0b000000000000001110,
    0b000000000000001110,
    0b000000000000001110,
    0b000000000000001110,
    0b000000000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b001111111111111100,
    0b000111111111111000,
    0b000001111111100000,
    0b000000000000000000,
]);
const GLYPH_K: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b011100000000011100,
    0b011100000000111000,
    0b011100000001110000,
    0b011100000011100000,
    0b011100001110000000,
    0b011100111000000000,
    0b011111100000000000,
    0b011111000000000000,
    0b011111100000000000,
    0b011100111000000000,
    0b011100001110000000,
    0b011100000011100000,
    0b011100000001110000,
    0b011100000000111000,
    0b011100000000011100,
    0b000000000000000000,
]);
const GLYPH_L: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011111111111111110,
    0b011111111111111110,
    0b011111111111111110,
    0b000000000000000000,
]);
const GLYPH_M: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b011100000000001110,
    0b011110000000011110,
    0b011111000000111110,
    0b011111100001111110,
    0b011101110011101110,
    0b011100111111001110,
    0b011100011110001110,
    0b011100001100001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b000000000000000000,
]);
const GLYPH_N: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b011100000000001110,
    0b011100000000001110,
    0b011110000000001110,
    0b011111000000001110,
    0b011111100000001110,
    0b011101110000001110,
    0b011100111000001110,
    0b011100011110001110,
    0b011100000111001110,
    0b011100000011101110,
    0b011100000001111110,
    0b011100000000111110,
    0b011100000000011110,
    0b011100000000001110,
    0b011100000000001110,
    0b000000000000000000,
]);
const GLYPH_O: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b000001111111100000,
    0b000111111111111000,
    0b001111111111111100,
    0b001110000000011100,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b001110000000011100,
    0b001111111111111100,
    0b000111111111111000,
    0b000001111111100000,
    0b000000000000000000,
]);
const GLYPH_P: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b011111111111100000,
    0b011111111111111000,
    0b011111111111111100,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011111111111111100,
    0b011111111111111000,
    0b011111111111100000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b000000000000000000,
]);
const GLYPH_Q: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b000001111111100000,
    0b000111111111111000,
    0b001111111111111100,
    0b001110000000011100,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000001101110,
    0b011100000001111110,
    0b001110000000111100,
    0b001111111111111110,
    0b000111111111111110,
    0b000001111111101110,
    0b000000000000000000,
]);
const GLYPH_R: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b011111111111100000,
    0b011111111111111000,
    0b011111111111111100,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011111111111111100,
    0b011111111111111000,
    0b011111111111100000,
    0b011100000001110000,
    0b011100000000111000,
    0b011100000000011100,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b000000000000000000,
]);
const GLYPH_S: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b000001111111100000,
    0b000111111111111000,
    0b001111111111111100,
    0b001110000000011100,
    0b011100000000000000,
    0b011100000000000000,
    0b011111111111111000,
    0b001111111111111100,
    0b000111111111111110,
    0b000000000000001110,
    0b000000000000001110,
    0b001110000000011100,
    0b001111111111111100,
    0b000111111111111000,
    0b000001111111100000,
    0b000000000000000000,
]);
const GLYPH_T: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b011111111111111110,
    0b011111111111111110,
    0b011111111111111110,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000011100000000,
    0b000000000000000000,
]);
const GLYPH_U: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b011100000000001110,
    0b001110000000011100,
    0b001111111111111100,
    0b000111111111111000,
    0b000001111111100000,
    0b000000000000000000,
]);
const GLYPH_V: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b011100000000001110,
    0b011100000000001110,
    0b001110000000011100,
    0b001110000000011100,
    0b000111000000111000,
    0b000111000000111000,
    0b000011100001110000,
    0b000011100001110000,
    0b000001110011100000,
    0b000001110011100000,
    0b000000111111000000,
    0b000000111111000000,
    0b000000011110000000,
    0b000000011110000000,
    0b000000001100000000,
    0b000000000000000000,
]);

const GLYPH_W: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b011100011100001110,
    0b011100011100001110,
    0b011100011100001110,
    0b011100011100001110,
    0b011100011100001110,
    0b001110111111011100,
    0b001110111111011100,
    0b001110111111011100,
    0b000111110011111000,
    0b000111110011111000,
    0b000111110011111000,
    0b000111110011111000,
    0b000011100001110000,
    0b000011100001110000,
    0b000001000000100000,
    0b000000000000000000,
]);
const GLYPH_X: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b011100000000001110,
    0b001110000000011100,
    0b000111000000111000,
    0b000011100001110000,
    0b000001110011100000,
    0b000000111111000000,
    0b000000011110000000,
    0b000000011110000000,
    0b000000011110000000,
    0b000000111111000000,
    0b000001110011100000,
    0b000011100001110000,
    0b000111000000111000,
    0b001110000000011100,
    0b011100000000001110,
    0b000000000000000000,
]);
const GLYPH_Y: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b011100000000001110,
    0b011100000000001110,
    0b001110000000011100,
    0b000111000000111000,
    0b000111000000111000,
    0b000011100001110000,
    0b000001110011100000,
    0b000000111111000000,
    0b000000011110000000,
    0b000000011110000000,
    0b000000011110000000,
    0b000000011110000000,
    0b000000011110000000,
    0b000000011110000000,
    0b000000011110000000,
    0b000000000000000000,
]);
const GLYPH_Z: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b011111111111111110,
    0b011111111111111110,
    0b011111111111111110,
    0b000000000000111100,
    0b000000000000111000,
    0b000000000011110000,
    0b000000000111100000,
    0b000000011110000000,
    0b000001111000000000,
    0b000011110000000000,
    0b000111100000000000,
    0b001111000000000000,
    0b011111111111111110,
    0b011111111111111110,
    0b011111111111111110,
    0b000000000000000000,
]);

const GLYPH_UP: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000001100000000,
    0b000000011110000000,
    0b000000111111000000,
    0b000001111111100000,
    0b000011111111110000,
    0b000111111111111000,
    0b001111011110111100,
    0b011110011110011110,
    0b011100011110001110,
    0b011000011110000110,
    0b010000011110000010,
    0b000000011110000000,
    0b000000011110000000,
    0b000000011110000000,
    0b000000011110000000,
    0b000000011110000000,
    0b000000000000000000,
]);
const GLYPH_DOWN: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000011110000000,
    0b000000011110000000,
    0b000000011110000000,
    0b000000011110000000,
    0b000000011110000000,
    0b010000011110000010,
    0b011000011110000110,
    0b011100011110001110,
    0b011110011110011110,
    0b001111011110111100,
    0b000111111111111000,
    0b000011111111110000,
    0b000001111111100000,
    0b000000111111000000,
    0b000000011110000000,
    0b000000001100000000,
    0b000000000000000000,
]);
const GLYPH_LEFT: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000001111000000,
    0b000000011110000000,
    0b000000111100000000,
    0b000001111000000000,
    0b000011110000000000,
    0b000111100000000000,
    0b001111111111111110,
    0b011111111111111110,
    0b011111111111111110,
    0b001111111111111110,
    0b000111100000000000,
    0b000011110000000000,
    0b000001111000000000,
    0b000000111100000000,
    0b000000011110000000,
    0b000000001111000000,
    0b000000000000000000,
]);
const GLYPH_RIGHT: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000111100000000,
    0b000000011110000000,
    0b000000001111000000,
    0b000000000111100000,
    0b000000000011110000,
    0b000000000001111000,
    0b011111111111111100,
    0b011111111111111110,
    0b011111111111111110,
    0b011111111111111100,
    0b000000000001111000,
    0b000000000011110000,
    0b000000000111100000,
    0b000000001111000000,
    0b000000011110000000,
    0b000000111100000000,
    0b000000000000000000,
]);
const GLYPH_UP_LEFT: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b011111111110000000,
    0b011111111111000000,
    0b011111111111100000,
    0b011111111111110000,
    0b011111111000000000,
    0b011111111100000000,
    0b011111111110000000,
    0b011111111111000000,
    0b011110111111100000,
    0b01111001111111000,
    0b001110001111111000,
    0b000110000111111100,
    0b000010000011111110,
    0b000000000001111110,
    0b000000000000111110,
    0b000000000000011110,
    0b000000000000000000,
]);
const GLYPH_UP_RIGHT: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000011111111110,
    0b000000111111111110,
    0b000001111111111110,
    0b000011111111111110,
    0b000000000111111110,
    0b000000001111111110,
    0b000000011111111110,
    0b000000111111111110,
    0b000001111111011110,
    0b000011111110011110,
    0b000111111100011100,
    0b001111111000011000,
    0b011111110000010000,
    0b011111100000000000,
    0b011111000000000000,
    0b011110000000000000,
    0b000000000000000000,
]);
const GLYPH_TURN_LEFT: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000001111000000000,
    0b000011110000000000,
    0b000111100000000000,
    0b001111111111110000,
    0b011111111111111100,
    0b001111111111111110,
    0b000111100000001110,
    0b000011110000001110,
    0b000001111000001110,
    0b000000000000001110,
    0b000000000000001110,
    0b000000000000001110,
    0b001110000000011100,
    0b001111111111111100,
    0b000111111111111000,
    0b000001111111100000,
    0b000000000000000000,
]);
const GLYPH_TURN_RIGHT: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000111100000,
    0b000000000011110000,
    0b000000000001111000,
    0b000011111111111100,
    0b001111111111111110,
    0b011111111111111100,
    0b011100000001111000,
    0b011100000011110000,
    0b011100000111100000,
    0b011100000000000000,
    0b011100000000000000,
    0b011100000000000000,
    0b001110000000011100,
    0b001111111111111100,
    0b000111111111111000,
    0b000001111111100000,
    0b000000000000000000,
]);

const GLYPH_YES: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b000000000000000000,
    0b010000000000000000,
    0b011000000000000000,
    0b011100000000000000,
    0b011110000000000110,
    0b001111000000011100,
    0b000111100001111000,
    0b000011110011110000,
    0b000001111111100000,
    0b000000111111000000,
    0b000000011110000000,
    0b000000001100000000,
    0b000000000000000000,
    0b000000000000000000,
    0b000000000000000000,
    0b000000000000000000,
]);
const GLYPH_NO: GlyphSource = GlyphSource::new([
    0b000000000000000000,
    0b000000000000000000,
    0b000000000000000000,
    0b000111000000111000,
    0b000111100001111000,
    0b000111110011111000,
    0b000011111111110000,
    0b000001111111100000,
    0b000000111111000000,
    0b000000111111000000,
    0b000001111111100000,
    0b000011111111110000,
    0b000111110011111000,
    0b000111100001111000,
    0b000111000000111000,
    0b000000000000000000,
    0b000000000000000000,
    0b000000000000000000,
]);

const fn glyph_bar_up(value: usize) -> GlyphSource {
    const V0: u32 = 0b000000000000000000;
    const V1: u32 = 0b001111111111111100;
    let bits = [
        if value > 17 { V1 } else { V0 },
        if value > 16 { V1 } else { V0 },
        if value > 15 { V1 } else { V0 },
        if value > 14 { V1 } else { V0 },
        if value > 13 { V1 } else { V0 },
        if value > 12 { V1 } else { V0 },
        if value > 11 { V1 } else { V0 },
        if value > 10 { V1 } else { V0 },
        if value > 9 { V1 } else { V0 },
        if value > 8 { V1 } else { V0 },
        if value > 7 { V1 } else { V0 },
        if value > 6 { V1 } else { V0 },
        if value > 5 { V1 } else { V0 },
        if value > 4 { V1 } else { V0 },
        if value > 3 { V1 } else { V0 },
        if value > 2 { V1 } else { V0 },
        if value > 1 { V1 } else { V0 },
        if value > 0 { V1 } else { V0 },
    ];
    GlyphSource { bits }
}

const fn glyph_bar_down(value: usize) -> GlyphSource {
    const V0: u32 = 0b000000000000000000;
    const V1: u32 = 0b001111111111111100;
    let bits = [
        if value > 0 { V1 } else { V0 },
        if value > 1 { V1 } else { V0 },
        if value > 2 { V1 } else { V0 },
        if value > 3 { V1 } else { V0 },
        if value > 4 { V1 } else { V0 },
        if value > 5 { V1 } else { V0 },
        if value > 6 { V1 } else { V0 },
        if value > 7 { V1 } else { V0 },
        if value > 8 { V1 } else { V0 },
        if value > 9 { V1 } else { V0 },
        if value > 10 { V1 } else { V0 },
        if value > 11 { V1 } else { V0 },
        if value > 12 { V1 } else { V0 },
        if value > 13 { V1 } else { V0 },
        if value > 14 { V1 } else { V0 },
        if value > 15 { V1 } else { V0 },
        if value > 16 { V1 } else { V0 },
        if value > 17 { V1 } else { V0 },
    ];
    GlyphSource { bits }
}

const fn glyph_bar_left(value: usize) -> GlyphSource {
    const FULL: u32 = 0b111111111111111111;
    let row = (FULL << (GLYPH_SIZE - value)) & FULL;
    let bits = [row; GLYPH_SIZE];
    GlyphSource { bits }
}

const fn glyph_bar_right(value: usize) -> GlyphSource {
    const FULL: u32 = 0b111111111111111111;
    let row = (FULL >> (GLYPH_SIZE - value)) & FULL;
    let bits = [row; GLYPH_SIZE];
    GlyphSource { bits }
}
