#![allow(unused_imports)]

use itertools::*;
use std::arch::x86_64::_pext_u64;
use std::cmp::Ordering;
use std::fmt;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

mod bitboards_generation {
    use super::*;

    #[derive(Clone, Copy, Default)]
    struct BitBoard(u64);

    impl fmt::Debug for BitBoard {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "BitBoard::new(0x{:X})", self.0)
        }
    }

    #[inline]
    const fn get_ranks_bb(index: usize) -> u64 {
        0xFF << (index << 3)
    }

    #[inline]
    const fn get_files_bb(index: usize) -> u64 {
        0x0101010101010101 << index
    }

    #[inline]
    const fn shift_up(bb: u64) -> u64 {
        (bb & !get_ranks_bb(7)) << 8
    }

    #[inline]
    const fn shift_down(bb: u64) -> u64 {
        bb >> 8
    }

    #[inline]
    const fn shift_left(bb: u64) -> u64 {
        (bb & !get_files_bb(0)) >> 1
    }

    #[inline]
    const fn shift_right(bb: u64) -> u64 {
        (bb & !get_files_bb(7)) << 1
    }

    #[inline]
    const fn get_rank(square: u8) -> u8 {
        square >> 3
    }

    #[inline]
    const fn get_file(square: u8) -> u8 {
        square & 7
    }

    #[inline]
    const fn wrapping_up(square: u8) -> u8 {
        if bb_contains(get_ranks_bb(7), square) {
            square
        } else {
            square + 8
        }
    }

    #[inline]
    const fn wrapping_down(square: u8) -> u8 {
        if bb_contains(get_ranks_bb(0), square) {
            square
        } else {
            square - 8
        }
    }

    #[inline]
    const fn wrapping_left(square: u8) -> u8 {
        if bb_contains(get_files_bb(0), square) {
            square
        } else {
            square - 1
        }
    }

    #[inline]
    const fn wrapping_right(square: u8) -> u8 {
        if bb_contains(get_files_bb(7), square) {
            square
        } else {
            square + 1
        }
    }

    const fn flip_horizontal(mut bb: u64) -> u64 {
        bb = ((bb >> 1) & 0x5555_5555_5555_5555) | ((bb & 0x5555_5555_5555_5555) << 1);
        bb = ((bb >> 2) & 0x3333_3333_3333_3333) | ((bb & 0x3333_3333_3333_3333) << 2);
        bb = ((bb >> 4) & 0x0F0F_0F0F_0F0F_0F0F) | ((bb & 0x0F0F_0F0F_0F0F_0F0F) << 4);
        bb
    }

    #[inline]
    const fn bb_contains(bb: u64, square: u8) -> bool {
        bb & 1 << square != 0
    }

    fn generate_moves_bb(sub_mask: u64, square: u8, piece_type: u8) -> u64 {
        let direction_offset = match piece_type {
            2 => 4,
            3 => 0,
            _ => unreachable!(),
        };
        let mut moves_bb = 0;
        for direction_index in 0..4 {
            let mut current_square_bb = 1 << square;
            loop {
                current_square_bb = match direction_index + direction_offset {
                    0 => shift_up(current_square_bb),
                    1 => shift_down(current_square_bb),
                    2 => shift_left(current_square_bb),
                    3 => shift_right(current_square_bb),
                    4 => shift_left(shift_up(current_square_bb)),
                    5 => shift_right(shift_up(current_square_bb)),
                    6 => shift_left(shift_down(current_square_bb)),
                    7 => shift_right(shift_down(current_square_bb)),
                    _ => unreachable!(),
                };
                moves_bb = moves_bb ^ current_square_bb;
                if current_square_bb == 0 || current_square_bb & sub_mask != 0 {
                    break;
                }
            }
        }
        moves_bb
    }

    fn generate_all_sub_masks_and_moves(
        mask: u64,
        square: u8,
        piece_type: u8,
    ) -> [(u64, u64); 1 << 12] {
        let mut array = [(0, 0); 1 << 12];

        // Generating Squares from BitBoard
        let mut squares = [None; 12];
        let mut pointer = 0;
        let mut mask_copy = mask;
        while mask_copy != 0 {
            let mask_square = mask_copy.trailing_zeros() as u8;
            squares[pointer] = Some(mask_square);
            pointer += 1;
            mask_copy = mask_copy ^ 1 << mask_square;
        }

        // Generate Sub Masks
        let mask_popcnt = mask.count_ones() as usize;
        for mask_index in 0..1 << mask_popcnt {
            for bit_index in 0..mask_popcnt {
                if bb_contains(mask_index, bit_index as u8) {
                    array[mask_index as usize].0 =
                        array[mask_index as usize].0 ^ 1 << squares[bit_index].unwrap();
                }
            }
            array[mask_index as usize].1 =
                generate_moves_bb(array[mask_index as usize].0, square, piece_type);
        }

        array
    }

    fn create_pawn_moves(file: &mut File) -> Result<()> {
        let mut moves_array = [[BitBoard::default(); 64]; 2];
        let mut attacks_array = [[BitBoard::default(); 64]; 2];
        for i in 0..2 {
            for j in 0..64 {
                moves_array[i][j].0 = if i == 0 {
                    shift_up(1 << j)
                } else {
                    shift_down(1 << j)
                };
                attacks_array[i][j].0 =
                    shift_left(moves_array[i][j].0) ^ shift_right(moves_array[i][j].0);
            }
        }
        for i in 0..8 {
            moves_array[0][i + 8].0 ^= shift_up(moves_array[0][i + 8].0);
            moves_array[1][i + 48].0 ^= shift_down(moves_array[1][i + 48].0);
        }

        writeln!(
            file,
            "const PAWN_MOVES_AND_ATTACKS: [[[BitBoard; 64]; 2]; 2] = {:#?};",
            [moves_array, attacks_array],
        )?;

        Ok(())
    }

    fn create_knight_moves(file: &mut File) -> Result<()> {
        writeln!(
            file,
            "const KNIGHT_MOVES: [BitBoard; 64] = {:#?};",
            std::array::from_fn::<_, 64, _>(|index| {
                let bb_square = 1 << index;
                let two_up_and_down =
                    shift_up(shift_up(bb_square)) ^ shift_down(shift_down(bb_square));
                let two_left_and_right =
                    shift_left(shift_left(bb_square)) ^ shift_right(shift_right(bb_square));
                BitBoard(
                    shift_left(two_up_and_down)
                        ^ shift_right(two_up_and_down)
                        ^ shift_up(two_left_and_right)
                        ^ shift_down(two_left_and_right),
                )
            })
        )?;
        Ok(())
    }

    fn create_king_moves(file: &mut File) -> Result<()> {
        writeln!(
            file,
            "const KING_MOVES: [BitBoard; 64] = {:#?};",
            std::array::from_fn::<_, 64, _>(|index| {
                let mut bb = 1 << index;
                bb ^= shift_left(bb) ^ shift_right(bb);
                bb ^= shift_down(bb) ^ shift_up(bb);
                BitBoard(bb ^ 1 << index)
            })
        )?;
        Ok(())
    }

    fn calculate_between(square1: u8, square2: u8, all_direction_rays: &[BitBoard]) -> u64 {
        if !bb_contains(all_direction_rays[square1 as usize].0, square2) {
            return 0;
        }

        let square1_rank = get_rank(square1);
        let square1_file = get_file(square1);
        let square2_rank = get_rank(square2);
        let square2_file = get_file(square2);

        let rank_ordering = square1_rank.cmp(&square2_rank);
        let file_ordering = square1_file.cmp(&square2_file);

        let mut bb = 0;
        let mut current_square = square1;
        loop {
            let mut next_square = match rank_ordering {
                Ordering::Less => wrapping_up(current_square),
                Ordering::Equal => current_square,
                Ordering::Greater => wrapping_down(current_square),
            };
            next_square = match file_ordering {
                Ordering::Less => wrapping_right(next_square),
                Ordering::Equal => next_square,
                Ordering::Greater => wrapping_left(next_square),
            };
            if next_square == square2 {
                return bb;
            }
            bb = bb ^ 1 << next_square;
            current_square = next_square;
        }
    }

    fn calculate_line(
        square1: u8,
        square2: u8,
        bishop_diagonal_rays: &[BitBoard],
        bishop_anti_diagonal_rays: &[BitBoard],
        all_direction_rays: &[BitBoard],
    ) -> u64 {
        if !bb_contains(all_direction_rays[square1 as usize].0, square2) {
            return 0;
        }

        let square2_bb = 1 << square2;
        let mut possible_line = get_ranks_bb(get_rank(square1) as usize);
        if possible_line & square2_bb != 0 {
            return possible_line;
        }
        possible_line = get_files_bb(get_file(square1) as usize);
        if possible_line & square2_bb != 0 {
            return possible_line;
        }
        possible_line = bishop_diagonal_rays[square1 as usize].0;
        if possible_line & square2_bb != 0 {
            return possible_line;
        }
        possible_line = bishop_anti_diagonal_rays[square1 as usize].0;
        if possible_line & square2_bb != 0 {
            return possible_line;
        }

        unreachable!();
    }

    fn create_rays(file: &mut File) -> Result<(Vec<BitBoard>, Vec<BitBoard>)> {
        let mut bishop_diagonal_rays = vec![BitBoard::default(); 64];
        for index in 0..64 {
            let square = index as u8;
            let square_rank_index = get_rank(square);
            let square_file_index = get_file(square);
            bishop_diagonal_rays[index] = {
                let mut bb = 0x8040201008040201;
                for _ in 0..square_rank_index.abs_diff(square_file_index) {
                    bb = if square_rank_index < square_file_index {
                        shift_right(bb)
                    } else {
                        shift_left(bb)
                    };
                }
                BitBoard(bb)
            };
        }

        // index ^ 7 is vertical mirror of a square
        let bishop_anti_diagonal_rays = (0..64)
            .map(|index| BitBoard(flip_horizontal(bishop_diagonal_rays[index ^ 7].0)))
            .collect_vec();
        let bishop_rays = (0..64)
            .map(|index| {
                BitBoard(bishop_diagonal_rays[index].0 ^ bishop_anti_diagonal_rays[index].0)
            })
            .collect_vec();
        let rook_rays = (0..64)
            .map(|index| {
                BitBoard(
                    get_ranks_bb(get_rank(index) as usize) ^ get_files_bb(get_file(index) as usize),
                )
            })
            .collect_vec();
        let all_direction_rays = (0..64)
            .map(|index| BitBoard(bishop_rays[index].0 ^ rook_rays[index].0))
            .collect_vec();

        let mut between = [[BitBoard::default(); 64]; 64];
        let mut line = [[BitBoard::default(); 64]; 64];
        for square1 in 0..64 {
            for square2 in 0..square1 {
                between[square1][square2] = BitBoard(calculate_between(
                    square1 as u8,
                    square2 as u8,
                    &all_direction_rays,
                ));
                between[square2][square1] = between[square1][square2];
                line[square1][square2] = BitBoard(calculate_line(
                    square1 as u8,
                    square2 as u8,
                    &bishop_diagonal_rays,
                    &bishop_anti_diagonal_rays,
                    &all_direction_rays,
                ));
                line[square2][square1] = line[square1][square2];
            }
        }

        writeln!(
            file,
            "const BISHOP_DIAGONAL_RAYS: [BitBoard; 64] = {:#?};",
            bishop_diagonal_rays
        )?;
        writeln!(
            file,
            "const BISHOP_ANTI_DIAGONAL_RAYS: [BitBoard; 64] = {:#?};",
            bishop_anti_diagonal_rays
        )?;
        writeln!(
            file,
            "const BISHOP_RAYS: [BitBoard; 64] = {:#?};",
            bishop_rays
        )?;
        writeln!(file, "const ROOK_RAYS: [BitBoard; 64] = {:#?};", rook_rays)?;
        writeln!(
            file,
            "const ALL_DIRECTION_RAYS: [BitBoard; 64] = {:#?};",
            all_direction_rays
        )?;
        writeln!(
            file,
            "const BETWEEN: [[BitBoard; 64]; 64] = {:#?};",
            between
        )?;
        writeln!(file, "const LINE: [[BitBoard; 64]; 64] = {:#?};", line)?;

        Ok((bishop_rays, rook_rays))
    }

    fn create_all_slider_moves(
        file: &mut File,
        bishop_rays: &[BitBoard],
        rook_rays: &[BitBoard],
    ) -> Result<()> {
        #[derive(Clone, Copy, Debug)]
        struct Magic {
            magic_number: u64,
            mask: BitBoard,
            offset: usize,
            right_shift: u8,
        }

        #[derive(Clone, Copy, Debug, Default)]
        struct BmiMagic {
            blockers_mask: BitBoard,
            offset: usize,
        }

        #[rustfmt::skip]
        let mut bishop_and_rook_magic_numbers = [[
            Magic { magic_number: 0x204022080a222040, mask: BitBoard(18049651735527936), offset: 0, right_shift: 58 },
            Magic { magic_number: 0x0020042400404100, mask: BitBoard(70506452091904), offset: 0, right_shift: 59 },
            Magic { magic_number: 0x421073004500023a, mask: BitBoard(275415828992), offset: 64, right_shift: 59 },
            Magic { magic_number: 0x0008048100401040, mask: BitBoard(1075975168), offset: 32, right_shift: 59 },
            Magic { magic_number: 0x8004042100840000, mask: BitBoard(38021120), offset: 96, right_shift: 59 },
            Magic { magic_number: 0x0001040240828006, mask: BitBoard(8657588224), offset: 64, right_shift: 59 },
            Magic { magic_number: 0x00818c0520300620, mask: BitBoard(2216338399232), offset: 128, right_shift: 59 },
            Magic { magic_number: 0x0a10210048200900, mask: BitBoard(567382630219776), offset: 96, right_shift: 58 },
            Magic { magic_number: 0x2090210202180100, mask: BitBoard(9024825867763712), offset: 0, right_shift: 59 },
            Magic { magic_number: 0x88050c1816004209, mask: BitBoard(18049651735527424), offset: 160, right_shift: 59 },
            Magic { magic_number: 0x88050c1816004209, mask: BitBoard(70506452221952), offset: 160, right_shift: 59 },
            Magic { magic_number: 0x0040040404840000, mask: BitBoard(275449643008), offset: 192, right_shift: 59 },
            Magic { magic_number: 0x0020021210402001, mask: BitBoard(9733406720), offset: 192, right_shift: 59 },
            Magic { magic_number: 0x0100110308400104, mask: BitBoard(2216342585344), offset: 224, right_shift: 59 },
            Magic { magic_number: 0x4c00424208244000, mask: BitBoard(567382630203392), offset: 224, right_shift: 59 },
            Magic { magic_number: 0x9100012202100c00, mask: BitBoard(1134765260406784), offset: 128, right_shift: 59 },
            Magic { magic_number: 0x881000400c534408, mask: BitBoard(4512412933816832), offset: 256, right_shift: 59 },
            Magic { magic_number: 0x2090210202180100, mask: BitBoard(9024825867633664), offset: 256, right_shift: 59 },
            Magic { magic_number: 0x8010004104108030, mask: BitBoard(18049651768822272), offset: 288, right_shift: 57 },
            Magic { magic_number: 0x2808000082004008, mask: BitBoard(70515108615168), offset: 288, right_shift: 57 },
            Magic { magic_number: 0x05010108200800a0, mask: BitBoard(2491752130560), offset: 416, right_shift: 57 },
            Magic { magic_number: 0x0280800101514000, mask: BitBoard(567383701868544), offset: 416, right_shift: 57 },
            Magic { magic_number: 0x02040101009a9000, mask: BitBoard(1134765256220672), offset: 544, right_shift: 59 },
            Magic { magic_number: 0x1000800100880130, mask: BitBoard(2269530512441344), offset: 544, right_shift: 59 },
            Magic { magic_number: 0x0020081250500100, mask: BitBoard(2256206450263040), offset: 576, right_shift: 59 },
            Magic { magic_number: 0x00082080840400a7, mask: BitBoard(4512412900526080), offset: 576, right_shift: 59 },
            Magic { magic_number: 0xa408880010182122, mask: BitBoard(9024834391117824), offset: 608, right_shift: 57 },
            Magic { magic_number: 0x0a00480004012020, mask: BitBoard(18051867805491712), offset: 608, right_shift: 55 },
            Magic { magic_number: 0x4c40840142802000, mask: BitBoard(637888545440768), offset: 736, right_shift: 55 },
            Magic { magic_number: 0x4089110002004104, mask: BitBoard(1135039602493440), offset: 1120, right_shift: 57 },
            Magic { magic_number: 0x02040101009a9000, mask: BitBoard(2269529440784384), offset: 1248, right_shift: 59 },
            Magic { magic_number: 0x0001020813108284, mask: BitBoard(4539058881568768), offset: 1248, right_shift: 59 },
            Magic { magic_number: 0x4014100400082001, mask: BitBoard(1128098963916800), offset: 1280, right_shift: 59 },
            Magic { magic_number: 0x010802820030a400, mask: BitBoard(2256197927833600), offset: 1280, right_shift: 59 },
            Magic { magic_number: 0x0004108802500840, mask: BitBoard(4514594912477184), offset: 1312, right_shift: 57 },
            Magic { magic_number: 0x00c0140400080211, mask: BitBoard(9592139778506752), offset: 1312, right_shift: 55 },
            Magic { magic_number: 0x24400100b0230040, mask: BitBoard(19184279556981248), offset: 1440, right_shift: 55 },
            Magic { magic_number: 0x8020040100022280, mask: BitBoard(2339762086609920), offset: 1824, right_shift: 57 },
            Magic { magic_number: 0x210d030100241400, mask: BitBoard(4538784537380864), offset: 1952, right_shift: 59 },
            Magic { magic_number: 0x1008010050002601, mask: BitBoard(9077569074761728), offset: 1952, right_shift: 59 },
            Magic { magic_number: 0x180404202a080401, mask: BitBoard(562958610993152), offset: 1984, right_shift: 59 },
            Magic { magic_number: 0x9800420231802000, mask: BitBoard(1125917221986304), offset: 1984, right_shift: 59 },
            Magic { magic_number: 0x000500148a015002, mask: BitBoard(2814792987328512), offset: 2016, right_shift: 57 },
            Magic { magic_number: 0x042044c010402200, mask: BitBoard(5629586008178688), offset: 2016, right_shift: 57 },
            Magic { magic_number: 0x00018a020a020c00, mask: BitBoard(11259172008099840), offset: 2144, right_shift: 57 },
            Magic { magic_number: 0x3001011001008080, mask: BitBoard(22518341868716544), offset: 2144, right_shift: 57 },
            Magic { magic_number: 0x8010844821408180, mask: BitBoard(9007336962655232), offset: 2272, right_shift: 59 },
            Magic { magic_number: 0xa004015605200601, mask: BitBoard(18014673925310464), offset: 2272, right_shift: 59 },
            Magic { magic_number: 0x00818c0520300620, mask: BitBoard(2216338399232), offset: 2304, right_shift: 59 },
            Magic { magic_number: 0x001442009008a009, mask: BitBoard(4432676798464), offset: 2304, right_shift: 59 },
            Magic { magic_number: 0x800d030080904002, mask: BitBoard(11064376819712), offset: 2336, right_shift: 59 },
            Magic { magic_number: 0x0008000104882182, mask: BitBoard(22137335185408), offset: 2336, right_shift: 59 },
            Magic { magic_number: 0x202000091024040c, mask: BitBoard(44272556441600), offset: 2368, right_shift: 59 },
            Magic { magic_number: 0x2000085010008080, mask: BitBoard(87995357200384), offset: 2368, right_shift: 59 },
            Magic { magic_number: 0x082092100200880a, mask: BitBoard(35253226045952), offset: 2400, right_shift: 59 },
            Magic { magic_number: 0x0020042400404100, mask: BitBoard(70506452091904), offset: 2400, right_shift: 59 },
            Magic { magic_number: 0x0a10210048200900, mask: BitBoard(567382630219776), offset: 2432, right_shift: 58 },
            Magic { magic_number: 0x9100012202100c00, mask: BitBoard(1134765260406784), offset: 2304, right_shift: 59 },
            Magic { magic_number: 0x0404242101c11040, mask: BitBoard(2832480465846272), offset: 2496, right_shift: 59 },
            Magic { magic_number: 0x4380800404208800, mask: BitBoard(5667157807464448), offset: 2432, right_shift: 59 },
            Magic { magic_number: 0x1000000040504100, mask: BitBoard(11333774449049600), offset: 2528, right_shift: 59 },
            Magic { magic_number: 0x400c04431c080084, mask: BitBoard(22526811443298304), offset: 2464, right_shift: 59 },
            Magic { magic_number: 0x2090210202180100, mask: BitBoard(9024825867763712), offset: 2400, right_shift: 59 },
            Magic { magic_number: 0x204022080a222040, mask: BitBoard(18049651735527936), offset: 2496, right_shift: 58 },
        ], [
            Magic { magic_number: 0x2280023020400080, mask: BitBoard(282578800148862), offset: 2560, right_shift: 52 },
            Magic { magic_number: 0x2840200010044000, mask: BitBoard(565157600297596), offset: 6656, right_shift: 53 },
            Magic { magic_number: 0x0880100009a00080, mask: BitBoard(1130315200595066), offset: 8704, right_shift: 53 },
            Magic { magic_number: 0x0080100014820800, mask: BitBoard(2260630401190006), offset: 10752, right_shift: 53 },
            Magic { magic_number: 0x0100030004080070, mask: BitBoard(4521260802379886), offset: 12800, right_shift: 53 },
            Magic { magic_number: 0x420006000130281c, mask: BitBoard(9042521604759646), offset: 14848, right_shift: 53 },
            Magic { magic_number: 0x0100008100240600, mask: BitBoard(18085043209519166), offset: 16896, right_shift: 53 },
            Magic { magic_number: 0x4080055123000080, mask: BitBoard(36170086419038334), offset: 18944, right_shift: 52 },
            Magic { magic_number: 0x600a002102004180, mask: BitBoard(282578800180736), offset: 23040, right_shift: 53 },
            Magic { magic_number: 0x600a002102004180, mask: BitBoard(565157600328704), offset: 25088, right_shift: 54 },
            Magic { magic_number: 0x4000801000200885, mask: BitBoard(1130315200625152), offset: 26112, right_shift: 54 },
            Magic { magic_number: 0xc001800802100080, mask: BitBoard(2260630401218048), offset: 27136, right_shift: 54 },
            Magic { magic_number: 0x0003001100880114, mask: BitBoard(4521260802403840), offset: 28160, right_shift: 54 },
            Magic { magic_number: 0x1282001200100429, mask: BitBoard(9042521604775424), offset: 29184, right_shift: 54 },
            Magic { magic_number: 0x01040006c8041011, mask: BitBoard(18085043209518592), offset: 30208, right_shift: 54 },
            Magic { magic_number: 0x030a000042188405, mask: BitBoard(36170086419037696), offset: 31232, right_shift: 53 },
            Magic { magic_number: 0x001080800040022a, mask: BitBoard(282578808340736), offset: 33280, right_shift: 53 },
            Magic { magic_number: 0x504840c00a201000, mask: BitBoard(565157608292864), offset: 35328, right_shift: 54 },
            Magic { magic_number: 0x00002a0010408200, mask: BitBoard(1130315208328192), offset: 36352, right_shift: 54 },
            Magic { magic_number: 0x4200a30010000900, mask: BitBoard(2260630408398848), offset: 37376, right_shift: 54 },
            Magic { magic_number: 0x0068010010080500, mask: BitBoard(4521260808540160), offset: 38400, right_shift: 54 },
            Magic { magic_number: 0x480080800a000400, mask: BitBoard(9042521608822784), offset: 39424, right_shift: 54 },
            Magic { magic_number: 0x40310400210a0810, mask: BitBoard(18085043209388032), offset: 40448, right_shift: 54 },
            Magic { magic_number: 0x0000020008810844, mask: BitBoard(36170086418907136), offset: 41472, right_shift: 53 },
            Magic { magic_number: 0x0080024040002010, mask: BitBoard(282580897300736), offset: 43520, right_shift: 53 },
            Magic { magic_number: 0x6080200880400180, mask: BitBoard(565159647117824), offset: 45568, right_shift: 54 },
            Magic { magic_number: 0x8000410100200210, mask: BitBoard(1130317180306432), offset: 46592, right_shift: 54 },
            Magic { magic_number: 0x1040100100210008, mask: BitBoard(2260632246683648), offset: 47616, right_shift: 54 },
            Magic { magic_number: 0x0008001100040900, mask: BitBoard(4521262379438080), offset: 48640, right_shift: 54 },
            Magic { magic_number: 0x0204040080800200, mask: BitBoard(9042522644946944), offset: 49664, right_shift: 54 },
            Magic { magic_number: 0x4001000100840200, mask: BitBoard(18085043175964672), offset: 50688, right_shift: 54 },
            Magic { magic_number: 0x0200c54200008409, mask: BitBoard(36170086385483776), offset: 51712, right_shift: 53 },
            Magic { magic_number: 0x0080006000400040, mask: BitBoard(283115671060736), offset: 53760, right_shift: 53 },
            Magic { magic_number: 0x600a002102004180, mask: BitBoard(565681586307584), offset: 55808, right_shift: 54 },
            Magic { magic_number: 0x4000403082002200, mask: BitBoard(1130822006735872), offset: 56832, right_shift: 54 },
            Magic { magic_number: 0x0012809000801804, mask: BitBoard(2261102847592448), offset: 57856, right_shift: 54 },
            Magic { magic_number: 0x5000800800800400, mask: BitBoard(4521664529305600), offset: 58880, right_shift: 54 },
            Magic { magic_number: 0x0204040080800200, mask: BitBoard(9042787892731904), offset: 59904, right_shift: 54 },
            Magic { magic_number: 0x20111008040005c2, mask: BitBoard(18085034619584512), offset: 60928, right_shift: 54 },
            Magic { magic_number: 0x0020542142000381, mask: BitBoard(36170077829103616), offset: 61952, right_shift: 53 },
            Magic { magic_number: 0x7000842040008000, mask: BitBoard(420017753620736), offset: 64000, right_shift: 53 },
            Magic { magic_number: 0x9010004020004000, mask: BitBoard(699298018886144), offset: 66048, right_shift: 54 },
            Magic { magic_number: 0x0002028040220010, mask: BitBoard(1260057572672512), offset: 67072, right_shift: 54 },
            Magic { magic_number: 0x1040100100210008, mask: BitBoard(2381576680245248), offset: 68096, right_shift: 54 },
            Magic { magic_number: 0x0088040008008080, mask: BitBoard(4624614895390720), offset: 69120, right_shift: 54 },
            Magic { magic_number: 0x0002000430420009, mask: BitBoard(9110691325681664), offset: 70144, right_shift: 54 },
            Magic { magic_number: 0x0040101881040002, mask: BitBoard(18082844186263552), offset: 71168, right_shift: 54 },
            Magic { magic_number: 0x2040240080420001, mask: BitBoard(36167887395782656), offset: 72192, right_shift: 53 },
            Magic { magic_number: 0x1802004108802200, mask: BitBoard(35466950888980736), offset: 74240, right_shift: 53 },
            Magic { magic_number: 0x6080200880400180, mask: BitBoard(34905104758997504), offset: 76288, right_shift: 54 },
            Magic { magic_number: 0x00002a0010408200, mask: BitBoard(34344362452452352), offset: 77312, right_shift: 54 },
            Magic { magic_number: 0xc001800802100080, mask: BitBoard(33222877839362048), offset: 78336, right_shift: 54 },
            Magic { magic_number: 0x0000180224008080, mask: BitBoard(30979908613181440), offset: 79360, right_shift: 54 },
            Magic { magic_number: 0x002e001004080a00, mask: BitBoard(26493970160820224), offset: 80384, right_shift: 54 },
            Magic { magic_number: 0x3021006a00040100, mask: BitBoard(17522093256097792), offset: 81408, right_shift: 54 },
            Magic { magic_number: 0x2a402d4104108200, mask: BitBoard(35607136465616896), offset: 82432, right_shift: 53 },
            Magic { magic_number: 0x0404402010800301, mask: BitBoard(9079539427579068672), offset: 84480, right_shift: 52 },
            Magic { magic_number: 0x0041001082204001, mask: BitBoard(8935706818303361536), offset: 88576, right_shift: 53 },
            Magic { magic_number: 0x5000501900422001, mask: BitBoard(8792156787827803136), offset: 90624, right_shift: 53 },
            Magic { magic_number: 0x0800182005005001, mask: BitBoard(8505056726876686336), offset: 92672, right_shift: 53 },
            Magic { magic_number: 0x0006001410592006, mask: BitBoard(7930856604974452736), offset: 94720, right_shift: 53 },
            Magic { magic_number: 0x0001006802140005, mask: BitBoard(6782456361169985536), offset: 96768, right_shift: 53 },
            Magic { magic_number: 0x1020080210028904, mask: BitBoard(4485655873561051136), offset: 98816, right_shift: 53 },
            Magic { magic_number: 0xc000192040840102, mask: BitBoard(9115426935197958144), offset: 100864, right_shift: 52 },
        ]];
        let mut bishop_and_rook_bmi_masks = [[BmiMagic::default(); 64]; 2];

        const NUM_MOVES: usize = 64 * (1 << 12) + 64 * (1 << 9);
        let mut moves = vec![BitBoard::default(); NUM_MOVES];
        let mut rays_cache_temp = vec![0; NUM_MOVES];
        let mut offset = 0;
        let mut bmi_offset = 0;
        let mut bmi_moves = vec![0; 107648];
        for piece_index in 0..2 {
            for square_index in 0..64 {
                let ray = match piece_index {
                    0 => bishop_rays,
                    1 => rook_rays,
                    _ => unreachable!(),
                }[square_index];

                let magic = &mut bishop_and_rook_magic_numbers[piece_index][square_index];
                magic.mask.0 = ray.0
                    & match piece_index {
                        0 => 0x007E7E7E7E7E7E00,
                        1 => {
                            let mut restriction = 0x007E7E7E7E7E7E00;
                            for (corner_rows, allowed) in const {
                                [
                                    (0x00000000000000FF, 0x000000000000007E),
                                    (0xFF00000000000000, 0x7E00000000000000),
                                    (0x0101010101010101, 0x0001010101010100),
                                    (0x8080808080808080, 0x0080808080808000),
                                ]
                            } {
                                if bb_contains(corner_rows, square_index as u8) {
                                    restriction ^= allowed;
                                }
                            }
                            restriction
                        }
                        _ => unreachable!(),
                    };
                magic.right_shift = 64 - magic.mask.0.count_ones() as u8;
                let sub_masks_and_moves_array = generate_all_sub_masks_and_moves(
                    magic.mask.0,
                    square_index as u8,
                    if piece_index == 0 { 2 } else { 3 },
                );
                let num_sub_masks = 1 << (64 - magic.right_shift);
                magic.offset = (0..offset)
                    .find(|&i| {
                        rays_cache_temp[i..i + num_sub_masks]
                            .iter()
                            .all(|&bb| bb & ray.0 == 0)
                    })
                    .unwrap_or(offset);
                offset = offset.max(magic.offset + num_sub_masks);

                let bmi_magic = &mut bishop_and_rook_bmi_masks[piece_index][square_index];
                bmi_magic.blockers_mask = magic.mask;
                bmi_magic.offset = bmi_offset;
                bmi_offset += num_sub_masks;

                for sub_masks_and_moves_array_index in 0..num_sub_masks {
                    let (sub_mask, moves_bb) =
                        sub_masks_and_moves_array[sub_masks_and_moves_array_index];
                    let index = (magic.magic_number.wrapping_mul(sub_mask) >> magic.right_shift)
                        as usize
                        + magic.offset;
                    moves[index].0 |= moves_bb;
                    rays_cache_temp[index] |= ray.0;

                    let sub_mask_key = unsafe {
                        _pext_u64(
                            sub_masks_and_moves_array[sub_masks_and_moves_array_index].0,
                            bmi_magic.blockers_mask.0,
                        ) as usize
                    };
                    bmi_moves[bmi_magic.offset + sub_mask_key] = unsafe {
                        _pext_u64(
                            sub_masks_and_moves_array[sub_masks_and_moves_array_index].1,
                            ray.0,
                        ) as u16
                    };
                }
            }
        }

        writeln!(file, r##"#[derive(Clone, Copy)]"##)?;
        writeln!(file, r##"struct Magic {{"##)?;
        writeln!(file, r##"    magic_number: u64,"##)?;
        writeln!(file, r##"    mask: BitBoard,"##)?;
        writeln!(file, r##"    offset: usize,"##)?;
        writeln!(file, r##"    right_shift: u8,"##)?;
        writeln!(file, r##"}}"##)?;

        writeln!(
            file,
            r"const BISHOP_AND_ROOK_MAGIC_NUMBERS: [[Magic; 64]; 2] = {:#?};",
            bishop_and_rook_magic_numbers
        )?;
        writeln!(
            file,
            r"const MOVES: [BitBoard; {}] = {:#?};",
            offset,
            &moves[0..offset]
        )?;

        writeln!(file, r##"#[derive(Clone, Copy)]"##)?;
        writeln!(file, r##"struct BmiMagic {{"##)?;
        writeln!(file, r##"    blockers_mask: BitBoard,"##)?;
        writeln!(file, r##"    offset: usize,"##)?;
        writeln!(file, r##"}}"##)?;

        writeln!(
            file,
            r"const BISHOP_AND_ROOK_BMI_MASKS: [[BmiMagic; 64]; 2] = {:#?};",
            bishop_and_rook_bmi_masks,
        )?;
        writeln!(file, r"const BMI_MOVES: [u16; 107648] = {:#?};", bmi_moves)?;

        Ok(())
    }

    pub fn create_magic_bitboards() -> Result<()> {
        let out_dir_string = std::env::var("OUT_DIR")?;
        let output_dir = Path::new(&out_dir_string);
        let magic_file_path = output_dir.join("magic.rs");

        if magic_file_path.exists() {
            fs::remove_file(&magic_file_path)?;
        }

        let mut file = File::create_new(magic_file_path)?;

        create_pawn_moves(&mut file)?;
        create_knight_moves(&mut file)?;
        create_king_moves(&mut file)?;

        let (bishop_rays, rook_rays) = create_rays(&mut file)?;
        create_all_slider_moves(&mut file, &bishop_rays, &rook_rays)?;

        Ok(())
    }
}

#[cfg(feature = "inbuilt_nnue")]
mod nnue_features {
    use super::*;

    const NNUE_FILE_NAME: &str = "nn-62ef826d1a6d.nnue";
    // const NNUE_FILE_NAME: &str = "nn-f7d87b7a1789.nnue";
    // const NNUE_FILE_NAME: &str = "nn-c3ca321c51c9.nnue";

    fn remove_nnue_file(nnue_path: &Path) -> Result<()> {
        if nnue_path.is_file() {
            let err_msg = format!(
                "Could not delete file {}!",
                nnue_path.to_str().ok_or("NNUE Path not found")?
            );
            std::fs::remove_file(nnue_path).map_err(|_| err_msg)?;
        }
        Ok(())
    }

    fn nnue_downloaded_correctly(nnue_path: &Path) -> Result<bool> {
        if !nnue_path.is_file() {
            return Ok(false);
        }
        let expected_hash_start = NNUE_FILE_NAME
            .strip_prefix("nn-")
            .unwrap()
            .strip_suffix(".nnue")
            .unwrap();
        let nnue_data = std::fs::read(nnue_path)?;
        let hash = sha256::digest(nnue_data.as_slice());
        Ok(hash.starts_with(expected_hash_start))
    }

    fn generate_nnue_file(nnue_file: &mut File) -> Result<()> {
        let url = format!("https://tests.stockfishchess.org/api/nn/{}", NNUE_FILE_NAME);
        let response = minreq::get(url).send()?;
        if response.status_code == 200 {
            nnue_file
                .write_all(response.as_bytes())
                .map_err(|_| "Could not copy NNUE file data to the nnue file!")?;
            Ok(())
        } else {
            Err(format!("Could not download NNUE file! Check your internet connection! Got response status code {}", response.status_code).into())
        }
    }

    fn check_and_download_nnue(nnue_dir: &PathBuf) -> Result<()> {
        if !nnue_dir.is_dir() {
            std::fs::create_dir_all(nnue_dir.clone())?;
        }
        let nnue_path = nnue_dir.join("nn.nnue");
        if !nnue_downloaded_correctly(&nnue_path)? {
            remove_nnue_file(&nnue_path)?;
            let mut nnue_file = File::create(nnue_path.clone())
                .map_err(|_| format!("Failed to create file at {:#?}", nnue_dir))?;
            println!("cargo:rerun-if-env-changed=DOCS_RS");
            println!("cargo:rerun-if-env-changed=NNUE_DOWNLOAD");
            if std::env::var("DOCS_RS").is_ok()
                || std::env::var("NNUE_DOWNLOAD") == Ok("PAUSE".to_string())
            {
                return Ok(());
            }
            match generate_nnue_file(&mut nnue_file) {
                Ok(_) => {
                    println!("cargo:rerun-if-changed={:#?}", nnue_path);
                }
                Err(err) => {
                    remove_nnue_file(&nnue_path)?;
                    return Err(err);
                }
            }
            if !nnue_downloaded_correctly(&nnue_path)? {
                return Err("File not downloaded correctly!".into());
            }
        }
        Ok(())
    }

    pub fn download_nnue() -> Result<()> {
        let output_dir = std::env::var("OUT_DIR")?;
        let output_nnue_dir = Path::new(&output_dir).join("nnue_dir");
        // Backing up nnue file in local cache directory to prevent downloading it multiple times
        let nnue_dir = dirs::cache_dir()
            .map(|path| path.join("timecat").join("nnue_dir"))
            .unwrap_or(output_nnue_dir.clone());
        match check_and_download_nnue(&nnue_dir) {
            Ok(_) => {
                if nnue_dir != output_nnue_dir {
                    std::fs::create_dir_all(output_nnue_dir.clone())?;
                    std::fs::copy(nnue_dir.join("nn.nnue"), output_nnue_dir.join("nn.nnue"))?;
                }
            }
            Err(err) => {
                if nnue_dir == output_nnue_dir {
                    return Err(err);
                } else {
                    check_and_download_nnue(&output_nnue_dir)?;
                }
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    bitboards_generation::create_magic_bitboards()?;
    #[cfg(feature = "inbuilt_nnue")]
    nnue_features::download_nnue()?;
    Ok(())
}
