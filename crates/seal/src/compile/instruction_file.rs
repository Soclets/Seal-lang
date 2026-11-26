//! Made by `make_instruction_file.py` from `bytecode.rs`'s `Instruction`; run it again after changing
//! `Instruction`, never edit this by hand. How each instruction is written into a `.sealc` file and read back
//! (D141, D153), and which of its fields are registers and jumps, for the verifier.
// Each instruction's fields are written and read in the order they're declared.
#![allow(clippy::too_many_lines, clippy::match_same_arms)]

use super::bytecode::{Instruction, Register};
use super::file::{FileError, Reader, Writer};

/// Writes an instruction: its number, then its fields.
pub fn write(instruction: Instruction, out: &mut Writer) {
    match instruction {
        Instruction::Move { to, from } => {
            out.u8(0);
            out.u16(to);
            out.u16(from);
        }
        Instruction::LoadSmall { to, value } => {
            out.u8(1);
            out.u16(to);
            out.u32(value.cast_unsigned());
        }
        Instruction::LoadConstant { to, constant } => {
            out.u8(2);
            out.u16(to);
            out.u32(constant);
        }
        Instruction::LoadText { to, text } => {
            out.u8(3);
            out.u16(to);
            out.u32(text);
        }
        Instruction::LoadPlainCase { to, case } => {
            out.u8(4);
            out.u16(to);
            out.u32(case);
        }
        Instruction::GetGlobal { to, global } => {
            out.u8(5);
            out.u16(to);
            out.u16(global);
        }
        Instruction::SetGlobal { global, from } => {
            out.u8(6);
            out.u16(global);
            out.u16(from);
        }
        Instruction::Retain { register } => {
            out.u8(7);
            out.u16(register);
        }
        Instruction::Release { register } => {
            out.u8(8);
            out.u16(register);
        }
        Instruction::AddWhole { to, left, right } => {
            out.u8(9);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::SubtractWhole { to, left, right } => {
            out.u8(10);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::MultiplyWhole { to, left, right } => {
            out.u8(11);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::DivideWhole { to, left, right } => {
            out.u8(12);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::DivideFloor { to, left, right } => {
            out.u8(13);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::DivideTrunc { to, left, right } => {
            out.u8(14);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::RemainderFloor { to, left, right } => {
            out.u8(15);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::RemainderTrunc { to, left, right } => {
            out.u8(16);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::AddWholeSmall { to, left, right } => {
            out.u8(17);
            out.u16(to);
            out.u16(left);
            out.u16(right.cast_unsigned());
        }
        Instruction::AddWholeConstant { to, left, constant } => {
            out.u8(18);
            out.u16(to);
            out.u16(left);
            out.u16(constant);
        }
        Instruction::MultiplyWholeConstant { to, left, constant } => {
            out.u8(19);
            out.u16(to);
            out.u16(left);
            out.u16(constant);
        }
        Instruction::DivideFloorConstant { to, left, constant } => {
            out.u8(20);
            out.u16(to);
            out.u16(left);
            out.u16(constant);
        }
        Instruction::RemainderFloorConstant { to, left, constant } => {
            out.u8(21);
            out.u16(to);
            out.u16(left);
            out.u16(constant);
        }
        Instruction::NegateWhole { to, from } => {
            out.u8(22);
            out.u16(to);
            out.u16(from);
        }
        Instruction::AddFloat { to, left, right } => {
            out.u8(23);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::SubtractFloat { to, left, right } => {
            out.u8(24);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::MultiplyFloat { to, left, right } => {
            out.u8(25);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::DivideFloat { to, left, right } => {
            out.u8(26);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::NegateFloat { to, from } => {
            out.u8(27);
            out.u16(to);
            out.u16(from);
        }
        Instruction::SquareRoot { to, from } => {
            out.u8(28);
            out.u16(to);
            out.u16(from);
        }
        Instruction::AddFloatConstant { to, left, constant } => {
            out.u8(29);
            out.u16(to);
            out.u16(left);
            out.u16(constant);
        }
        Instruction::MultiplyFloatConstant { to, left, constant } => {
            out.u8(30);
            out.u16(to);
            out.u16(left);
            out.u16(constant);
        }
        Instruction::DivideFloatConstant { to, left, constant } => {
            out.u8(31);
            out.u16(to);
            out.u16(left);
            out.u16(constant);
        }
        Instruction::DivideConstantByFloat { to, constant, right } => {
            out.u8(32);
            out.u16(to);
            out.u16(constant);
            out.u16(right);
        }
        Instruction::EqualWhole { to, left, right } => {
            out.u8(33);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::NotEqualWhole { to, left, right } => {
            out.u8(34);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::LessWhole { to, left, right } => {
            out.u8(35);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::AtMostWhole { to, left, right } => {
            out.u8(36);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::FloatFused { shape, ops, to, a, b, c, d } => {
            out.u8(37);
            out.u8(shape);
            out.u8(ops);
            out.u8(to);
            out.u8(a);
            out.u8(b);
            out.u8(c);
            out.u8(d);
        }
        Instruction::PlaceOnly => {
            out.u8(38);
        }
        Instruction::UnpackItemAtPosition { base, list, index } => {
            out.u8(39);
            out.u16(base);
            out.u16(list);
            out.u16(index);
        }
        Instruction::UnpackItemAtOffset { base, list, index } => {
            out.u8(40);
            out.u16(base);
            out.u16(list);
            out.u16(index);
        }
        Instruction::ItemFieldAtPosition { to, list, index, field } => {
            out.u8(41);
            out.u16(to);
            out.u16(list);
            out.u16(index);
            out.u8(field);
        }
        Instruction::ItemFieldAtOffset { to, list, index, field } => {
            out.u8(42);
            out.u16(to);
            out.u16(list);
            out.u16(index);
            out.u8(field);
        }
        Instruction::SetItemFieldAtPosition { list, index, field, value } => {
            out.u8(43);
            out.u16(list);
            out.u16(index);
            out.u8(field);
            out.u16(value);
        }
        Instruction::SetItemFieldAtOffset { list, index, field, value } => {
            out.u8(44);
            out.u16(list);
            out.u16(index);
            out.u8(field);
            out.u16(value);
        }
        Instruction::FitList { to, from, limit } => {
            out.u8(45);
            out.u16(to);
            out.u16(from);
            out.u16(limit);
        }
        Instruction::Own { register } => {
            out.u8(46);
            out.u16(register);
        }
        Instruction::OwnField { to, record, field } => {
            out.u8(47);
            out.u16(to);
            out.u16(record);
            out.u16(field);
        }
        Instruction::OwnItemAtPosition { to, list, index } => {
            out.u8(48);
            out.u16(to);
            out.u16(list);
            out.u16(index);
        }
        Instruction::OwnItemAtOffset { to, list, index } => {
            out.u8(49);
            out.u16(to);
            out.u16(list);
            out.u16(index);
        }
        Instruction::TextLength { to, from, unit } => {
            out.u8(50);
            out.u16(to);
            out.u16(from);
            out.u8(unit);
        }
        Instruction::GraphemeAtPosition { to, text, index } => {
            out.u8(51);
            out.u16(to);
            out.u16(text);
            out.u16(index);
        }
        Instruction::GraphemeAtOffset { to, text, index } => {
            out.u8(52);
            out.u16(to);
            out.u16(text);
            out.u16(index);
        }
        Instruction::Graphemes { to, from } => {
            out.u8(53);
            out.u16(to);
            out.u16(from);
        }
        Instruction::SplitText { to, text, by } => {
            out.u8(54);
            out.u16(to);
            out.u16(text);
            out.u16(by);
        }
        Instruction::SplitTextNoEmpty { to, text, by } => {
            out.u8(55);
            out.u16(to);
            out.u16(text);
            out.u16(by);
        }
        Instruction::EqualText { to, left, right } => {
            out.u8(56);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::NotEqualText { to, left, right } => {
            out.u8(57);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::EqualFloat { to, left, right } => {
            out.u8(58);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::NotEqualFloat { to, left, right } => {
            out.u8(59);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::LessFloat { to, left, right } => {
            out.u8(60);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::AtMostFloat { to, left, right } => {
            out.u8(61);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::Not { to, from } => {
            out.u8(62);
            out.u16(to);
            out.u16(from);
        }
        Instruction::Jump { offset } => {
            out.u8(63);
            out.u32(offset.cast_unsigned());
        }
        Instruction::JumpIfTrue { register, offset } => {
            out.u8(64);
            out.u16(register);
            out.u32(offset.cast_unsigned());
        }
        Instruction::JumpIfFalse { register, offset } => {
            out.u8(65);
            out.u16(register);
            out.u32(offset.cast_unsigned());
        }
        Instruction::JumpIfEqualWhole { left, right, offset } => {
            out.u8(66);
            out.u16(left);
            out.u16(right);
            out.u16(offset.cast_unsigned());
        }
        Instruction::JumpIfNotEqualWhole { left, right, offset } => {
            out.u8(67);
            out.u16(left);
            out.u16(right);
            out.u16(offset.cast_unsigned());
        }
        Instruction::JumpIfLessWhole { left, right, offset } => {
            out.u8(68);
            out.u16(left);
            out.u16(right);
            out.u16(offset.cast_unsigned());
        }
        Instruction::JumpIfAtMostWhole { left, right, offset } => {
            out.u8(69);
            out.u16(left);
            out.u16(right);
            out.u16(offset.cast_unsigned());
        }
        Instruction::JumpIfEqualWholeSmall { left, right, offset } => {
            out.u8(70);
            out.u16(left);
            out.u16(right.cast_unsigned());
            out.u16(offset.cast_unsigned());
        }
        Instruction::JumpIfNotEqualWholeSmall { left, right, offset } => {
            out.u8(71);
            out.u16(left);
            out.u16(right.cast_unsigned());
            out.u16(offset.cast_unsigned());
        }
        Instruction::JumpIfLessWholeSmall { left, right, offset } => {
            out.u8(72);
            out.u16(left);
            out.u16(right.cast_unsigned());
            out.u16(offset.cast_unsigned());
        }
        Instruction::JumpIfAtMostWholeSmall { left, right, offset } => {
            out.u8(73);
            out.u16(left);
            out.u16(right.cast_unsigned());
            out.u16(offset.cast_unsigned());
        }
        Instruction::JumpIfGreaterWholeSmall { left, right, offset } => {
            out.u8(74);
            out.u16(left);
            out.u16(right.cast_unsigned());
            out.u16(offset.cast_unsigned());
        }
        Instruction::JumpIfAtLeastWholeSmall { left, right, offset } => {
            out.u8(75);
            out.u16(left);
            out.u16(right.cast_unsigned());
            out.u16(offset.cast_unsigned());
        }
        Instruction::JumpIfEqualFloat { left, right, offset } => {
            out.u8(76);
            out.u16(left);
            out.u16(right);
            out.u16(offset.cast_unsigned());
        }
        Instruction::JumpIfNotEqualFloat { left, right, offset } => {
            out.u8(77);
            out.u16(left);
            out.u16(right);
            out.u16(offset.cast_unsigned());
        }
        Instruction::JumpIfLessFloat { left, right, offset } => {
            out.u8(78);
            out.u16(left);
            out.u16(right);
            out.u16(offset.cast_unsigned());
        }
        Instruction::JumpIfAtMostFloat { left, right, offset } => {
            out.u8(79);
            out.u16(left);
            out.u16(right);
            out.u16(offset.cast_unsigned());
        }
        Instruction::Call { base, procedure } => {
            out.u8(80);
            out.u16(base);
            out.u32(procedure);
        }
        Instruction::CallAdding { base, procedure, from, add } => {
            out.u8(81);
            out.u16(base);
            out.u16(procedure);
            out.u16(from);
            out.u8(add.cast_unsigned());
        }
        Instruction::Return { from } => {
            out.u8(82);
            out.u16(from);
        }
        Instruction::ReturnIfWholeSmall { from, left, right, compare } => {
            out.u8(83);
            out.u16(from);
            out.u16(left);
            out.u16(right.cast_unsigned());
            out.u8(compare as u8);
        }
        Instruction::ReturnAddWhole { left, right } => {
            out.u8(84);
            out.u16(left);
            out.u16(right);
        }
        Instruction::ReturnNothing => {
            out.u8(85);
        }
        Instruction::CallDoor { base, site } => {
            out.u8(86);
            out.u16(base);
            out.u32(site);
        }
        Instruction::Fail { failure } => {
            out.u8(87);
            out.u16(failure);
        }
        Instruction::PassFailureUp => {
            out.u8(88);
        }
        Instruction::JumpUnlessFailed { offset } => {
            out.u8(89);
            out.u32(offset.cast_unsigned());
        }
        Instruction::JumpUnlessFailureIs { failure, offset } => {
            out.u8(90);
            out.u16(failure);
            out.u32(offset.cast_unsigned());
        }
        Instruction::ClearFailure => {
            out.u8(91);
        }
        Instruction::Crash => {
            out.u8(92);
        }
        Instruction::EqualValues { to, left, site } => {
            out.u8(93);
            out.u16(to);
            out.u16(left);
            out.u16(site);
        }
        Instruction::MakeRange { to, base, inclusive } => {
            out.u8(94);
            out.u16(to);
            out.u16(base);
            out.u8(u8::from(inclusive));
        }
        Instruction::PowerWhole { to, left, right } => {
            out.u8(95);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::PowerFloat { to, left, right } => {
            out.u8(96);
            out.u16(to);
            out.u16(left);
            out.u16(right);
        }
        Instruction::Maths { to, base, function } => {
            out.u8(97);
            out.u16(to);
            out.u16(base);
            out.u8(function);
        }
        Instruction::LoadNumber { to, site } => {
            out.u8(98);
            out.u16(to);
            out.u16(site);
        }
        Instruction::NumberOperation { to, left, right, operation } => {
            out.u8(99);
            out.u16(to);
            out.u16(left);
            out.u16(right);
            out.u8(operation);
        }
        Instruction::NumberUpdate { target, right, operation } => {
            out.u8(100);
            out.u16(target);
            out.u16(right);
            out.u8(operation);
        }
        Instruction::ConvertNumber { to, from, site } => {
            out.u8(101);
            out.u16(to);
            out.u16(from);
            out.u16(site);
        }
        Instruction::CheckLimits { register, site } => {
            out.u8(102);
            out.u16(register);
            out.u16(site);
        }
        Instruction::Scale { to, from, site } => {
            out.u8(103);
            out.u16(to);
            out.u16(from);
            out.u16(site);
        }
        Instruction::CrashIfNone { number } => {
            out.u8(104);
            out.u16(number);
        }
        Instruction::JoinPieces { to, site } => {
            out.u8(105);
            out.u16(to);
            out.u32(site);
        }
        Instruction::AppendPieces { to, site } => {
            out.u8(106);
            out.u16(to);
            out.u32(site);
        }
        Instruction::TextToBool { to, from, failure } => {
            out.u8(107);
            out.u16(to);
            out.u16(from);
            out.u16(failure);
        }
        Instruction::TextToWhole { to, from } => {
            out.u8(108);
            out.u16(to);
            out.u16(from);
        }
        Instruction::WholeToFloat { to, from } => {
            out.u8(109);
            out.u16(to);
            out.u16(from);
        }
        Instruction::FloatToWhole { to, from, rounding } => {
            out.u8(110);
            out.u16(to);
            out.u16(from);
            out.u16(rounding);
        }
        Instruction::NewList { to, base, count } => {
            out.u8(111);
            out.u16(to);
            out.u16(base);
            out.u16(count);
        }
        Instruction::NewReferenceList { to, base, count } => {
            out.u8(112);
            out.u16(to);
            out.u16(base);
            out.u16(count);
        }
        Instruction::RepeatList { to, value, count } => {
            out.u8(113);
            out.u16(to);
            out.u16(value);
            out.u16(count);
        }
        Instruction::RepeatReferenceList { to, value, count } => {
            out.u8(114);
            out.u16(to);
            out.u16(value);
            out.u16(count);
        }
        Instruction::ItemAtPosition { to, list, index } => {
            out.u8(115);
            out.u16(to);
            out.u16(list);
            out.u16(index);
        }
        Instruction::ItemAtOffset { to, list, index } => {
            out.u8(116);
            out.u16(to);
            out.u16(list);
            out.u16(index);
        }
        Instruction::SetItemAtPosition { list, index, value } => {
            out.u8(117);
            out.u16(list);
            out.u16(index);
            out.u16(value);
        }
        Instruction::SetItemAtOffset { list, index, value } => {
            out.u8(118);
            out.u16(list);
            out.u16(index);
            out.u16(value);
        }
        Instruction::AddItem { list, value } => {
            out.u8(119);
            out.u16(list);
            out.u16(value);
        }
        Instruction::InsertItemAtPosition { list, index, value } => {
            out.u8(120);
            out.u16(list);
            out.u16(index);
            out.u16(value);
        }
        Instruction::InsertItemAtOffset { list, index, value } => {
            out.u8(121);
            out.u16(list);
            out.u16(index);
            out.u16(value);
        }
        Instruction::CheckRoom { list, limit } => {
            out.u8(122);
            out.u16(list);
            out.u16(limit);
        }
        Instruction::RemoveItemAtPosition { list, index } => {
            out.u8(123);
            out.u16(list);
            out.u16(index);
        }
        Instruction::RemoveItemAtOffset { list, index } => {
            out.u8(124);
            out.u16(list);
            out.u16(index);
        }
        Instruction::Length { to, from } => {
            out.u8(125);
            out.u16(to);
            out.u16(from);
        }
        Instruction::NextItem { to, base, offset } => {
            out.u8(126);
            out.u16(to);
            out.u16(base);
            out.u16(offset.cast_unsigned());
        }
        Instruction::PrepareRangeTo { base } => {
            out.u8(127);
            out.u16(base);
        }
        Instruction::PrepareRangeBefore { base } => {
            out.u8(128);
            out.u16(base);
        }
        Instruction::NextInRange { to, base, offset } => {
            out.u8(129);
            out.u16(to);
            out.u16(base);
            out.u16(offset.cast_unsigned());
        }
        Instruction::NewDict { to, shape } => {
            out.u8(130);
            out.u16(to);
            out.u16(shape);
        }
        Instruction::GetKey { to, dict, key } => {
            out.u8(131);
            out.u16(to);
            out.u16(dict);
            out.u16(key);
        }
        Instruction::SetKey { dict, key, value } => {
            out.u8(132);
            out.u16(dict);
            out.u16(key);
            out.u16(value);
        }
        Instruction::RemoveKey { dict, key } => {
            out.u8(133);
            out.u16(dict);
            out.u16(key);
        }
        Instruction::HasKey { to, dict, key } => {
            out.u8(134);
            out.u16(to);
            out.u16(dict);
            out.u16(key);
        }
        Instruction::FindKey { to, dict, key } => {
            out.u8(135);
            out.u16(to);
            out.u16(dict);
            out.u16(key);
        }
        Instruction::DivideFloorByPowerOfTwo { to, left, shift } => {
            out.u8(136);
            out.u16(to);
            out.u16(left);
            out.u16(shift);
        }
        Instruction::RemainderFloorByPowerOfTwo { to, left, bits } => {
            out.u8(137);
            out.u16(to);
            out.u16(left);
            out.u16(bits);
        }
        Instruction::ReleaseText { register } => {
            out.u8(138);
            out.u16(register);
        }
        Instruction::GetKeyIfAny { to, dict, key } => {
            out.u8(139);
            out.u16(to);
            out.u16(dict);
            out.u16(key);
        }
        Instruction::GetFoundIfAny { to, dict, found } => {
            out.u8(140);
            out.u16(to);
            out.u16(dict);
            out.u16(found);
        }
        Instruction::GetFound { to, dict, found } => {
            out.u8(141);
            out.u16(to);
            out.u16(dict);
            out.u16(found);
        }
        Instruction::SetFound { dict, found, key } => {
            out.u8(142);
            out.u16(dict);
            out.u16(found);
            out.u16(key);
        }
        Instruction::NextEntry { to, base, offset } => {
            out.u8(143);
            out.u16(to);
            out.u16(base);
            out.u16(offset.cast_unsigned());
        }
        Instruction::NewRecord { to, base, layout } => {
            out.u8(144);
            out.u16(to);
            out.u16(base);
            out.u16(layout);
        }
        Instruction::GetField { to, record, field } => {
            out.u8(145);
            out.u16(to);
            out.u16(record);
            out.u16(field);
        }
        Instruction::SetField { record, field, value } => {
            out.u8(146);
            out.u16(record);
            out.u16(field);
            out.u16(value);
        }
        Instruction::SetReferenceField { record, field, value } => {
            out.u8(147);
            out.u16(record);
            out.u16(field);
            out.u16(value);
        }
        Instruction::NewCase { to, base, layout } => {
            out.u8(148);
            out.u16(to);
            out.u16(base);
            out.u16(layout);
        }
        Instruction::UnpackRecord { base, record } => {
            out.u8(149);
            out.u16(base);
            out.u16(record);
        }
        Instruction::PackIntoItemAtPosition { list, index, base } => {
            out.u8(150);
            out.u16(list);
            out.u16(index);
            out.u16(base);
        }
        Instruction::PackIntoItemAtOffset { list, index, base } => {
            out.u8(151);
            out.u16(list);
            out.u16(index);
            out.u16(base);
        }
        Instruction::CaseNumber { to, from } => {
            out.u8(152);
            out.u16(to);
            out.u16(from);
        }
        Instruction::GetCaseValue { to, case, field } => {
            out.u8(153);
            out.u16(to);
            out.u16(case);
            out.u16(field);
        }
        Instruction::Unreachable => {
            out.u8(154);
        }
    }
}

/// Reads an instruction back.
///
/// # Errors
///
/// When the file ends, or the number isn't an instruction's.
pub fn read(input: &mut Reader) -> Result<Instruction, FileError> {
    Ok(match input.u8()? {
        0 => Instruction::Move { to: input.u16()?, from: input.u16()? },
        1 => Instruction::LoadSmall { to: input.u16()?, value: input.u32()?.cast_signed() },
        2 => Instruction::LoadConstant { to: input.u16()?, constant: input.u32()? },
        3 => Instruction::LoadText { to: input.u16()?, text: input.u32()? },
        4 => Instruction::LoadPlainCase { to: input.u16()?, case: input.u32()? },
        5 => Instruction::GetGlobal { to: input.u16()?, global: input.u16()? },
        6 => Instruction::SetGlobal { global: input.u16()?, from: input.u16()? },
        7 => Instruction::Retain { register: input.u16()? },
        8 => Instruction::Release { register: input.u16()? },
        9 => Instruction::AddWhole { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        10 => Instruction::SubtractWhole { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        11 => Instruction::MultiplyWhole { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        12 => Instruction::DivideWhole { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        13 => Instruction::DivideFloor { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        14 => Instruction::DivideTrunc { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        15 => Instruction::RemainderFloor { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        16 => Instruction::RemainderTrunc { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        17 => Instruction::AddWholeSmall { to: input.u16()?, left: input.u16()?, right: input.u16()?.cast_signed() },
        18 => Instruction::AddWholeConstant { to: input.u16()?, left: input.u16()?, constant: input.u16()? },
        19 => Instruction::MultiplyWholeConstant { to: input.u16()?, left: input.u16()?, constant: input.u16()? },
        20 => Instruction::DivideFloorConstant { to: input.u16()?, left: input.u16()?, constant: input.u16()? },
        21 => Instruction::RemainderFloorConstant { to: input.u16()?, left: input.u16()?, constant: input.u16()? },
        22 => Instruction::NegateWhole { to: input.u16()?, from: input.u16()? },
        23 => Instruction::AddFloat { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        24 => Instruction::SubtractFloat { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        25 => Instruction::MultiplyFloat { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        26 => Instruction::DivideFloat { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        27 => Instruction::NegateFloat { to: input.u16()?, from: input.u16()? },
        28 => Instruction::SquareRoot { to: input.u16()?, from: input.u16()? },
        29 => Instruction::AddFloatConstant { to: input.u16()?, left: input.u16()?, constant: input.u16()? },
        30 => Instruction::MultiplyFloatConstant { to: input.u16()?, left: input.u16()?, constant: input.u16()? },
        31 => Instruction::DivideFloatConstant { to: input.u16()?, left: input.u16()?, constant: input.u16()? },
        32 => Instruction::DivideConstantByFloat { to: input.u16()?, constant: input.u16()?, right: input.u16()? },
        33 => Instruction::EqualWhole { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        34 => Instruction::NotEqualWhole { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        35 => Instruction::LessWhole { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        36 => Instruction::AtMostWhole { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        37 => Instruction::FloatFused {
            shape: input.u8()?,
            ops: input.u8()?,
            to: input.u8()?,
            a: input.u8()?,
            b: input.u8()?,
            c: input.u8()?,
            d: input.u8()?,
        },
        38 => Instruction::PlaceOnly,
        39 => Instruction::UnpackItemAtPosition { base: input.u16()?, list: input.u16()?, index: input.u16()? },
        40 => Instruction::UnpackItemAtOffset { base: input.u16()?, list: input.u16()?, index: input.u16()? },
        41 => Instruction::ItemFieldAtPosition {
            to: input.u16()?,
            list: input.u16()?,
            index: input.u16()?,
            field: input.u8()?,
        },
        42 => Instruction::ItemFieldAtOffset {
            to: input.u16()?,
            list: input.u16()?,
            index: input.u16()?,
            field: input.u8()?,
        },
        43 => Instruction::SetItemFieldAtPosition {
            list: input.u16()?,
            index: input.u16()?,
            field: input.u8()?,
            value: input.u16()?,
        },
        44 => Instruction::SetItemFieldAtOffset {
            list: input.u16()?,
            index: input.u16()?,
            field: input.u8()?,
            value: input.u16()?,
        },
        45 => Instruction::FitList { to: input.u16()?, from: input.u16()?, limit: input.u16()? },
        46 => Instruction::Own { register: input.u16()? },
        47 => Instruction::OwnField { to: input.u16()?, record: input.u16()?, field: input.u16()? },
        48 => Instruction::OwnItemAtPosition { to: input.u16()?, list: input.u16()?, index: input.u16()? },
        49 => Instruction::OwnItemAtOffset { to: input.u16()?, list: input.u16()?, index: input.u16()? },
        50 => Instruction::TextLength { to: input.u16()?, from: input.u16()?, unit: input.u8()? },
        51 => Instruction::GraphemeAtPosition { to: input.u16()?, text: input.u16()?, index: input.u16()? },
        52 => Instruction::GraphemeAtOffset { to: input.u16()?, text: input.u16()?, index: input.u16()? },
        53 => Instruction::Graphemes { to: input.u16()?, from: input.u16()? },
        54 => Instruction::SplitText { to: input.u16()?, text: input.u16()?, by: input.u16()? },
        55 => Instruction::SplitTextNoEmpty { to: input.u16()?, text: input.u16()?, by: input.u16()? },
        56 => Instruction::EqualText { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        57 => Instruction::NotEqualText { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        58 => Instruction::EqualFloat { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        59 => Instruction::NotEqualFloat { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        60 => Instruction::LessFloat { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        61 => Instruction::AtMostFloat { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        62 => Instruction::Not { to: input.u16()?, from: input.u16()? },
        63 => Instruction::Jump { offset: input.u32()?.cast_signed() },
        64 => Instruction::JumpIfTrue { register: input.u16()?, offset: input.u32()?.cast_signed() },
        65 => Instruction::JumpIfFalse { register: input.u16()?, offset: input.u32()?.cast_signed() },
        66 => Instruction::JumpIfEqualWhole {
            left: input.u16()?,
            right: input.u16()?,
            offset: input.u16()?.cast_signed(),
        },
        67 => Instruction::JumpIfNotEqualWhole {
            left: input.u16()?,
            right: input.u16()?,
            offset: input.u16()?.cast_signed(),
        },
        68 => {
            Instruction::JumpIfLessWhole { left: input.u16()?, right: input.u16()?, offset: input.u16()?.cast_signed() }
        }
        69 => Instruction::JumpIfAtMostWhole {
            left: input.u16()?,
            right: input.u16()?,
            offset: input.u16()?.cast_signed(),
        },
        70 => Instruction::JumpIfEqualWholeSmall {
            left: input.u16()?,
            right: input.u16()?.cast_signed(),
            offset: input.u16()?.cast_signed(),
        },
        71 => Instruction::JumpIfNotEqualWholeSmall {
            left: input.u16()?,
            right: input.u16()?.cast_signed(),
            offset: input.u16()?.cast_signed(),
        },
        72 => Instruction::JumpIfLessWholeSmall {
            left: input.u16()?,
            right: input.u16()?.cast_signed(),
            offset: input.u16()?.cast_signed(),
        },
        73 => Instruction::JumpIfAtMostWholeSmall {
            left: input.u16()?,
            right: input.u16()?.cast_signed(),
            offset: input.u16()?.cast_signed(),
        },
        74 => Instruction::JumpIfGreaterWholeSmall {
            left: input.u16()?,
            right: input.u16()?.cast_signed(),
            offset: input.u16()?.cast_signed(),
        },
        75 => Instruction::JumpIfAtLeastWholeSmall {
            left: input.u16()?,
            right: input.u16()?.cast_signed(),
            offset: input.u16()?.cast_signed(),
        },
        76 => Instruction::JumpIfEqualFloat {
            left: input.u16()?,
            right: input.u16()?,
            offset: input.u16()?.cast_signed(),
        },
        77 => Instruction::JumpIfNotEqualFloat {
            left: input.u16()?,
            right: input.u16()?,
            offset: input.u16()?.cast_signed(),
        },
        78 => {
            Instruction::JumpIfLessFloat { left: input.u16()?, right: input.u16()?, offset: input.u16()?.cast_signed() }
        }
        79 => Instruction::JumpIfAtMostFloat {
            left: input.u16()?,
            right: input.u16()?,
            offset: input.u16()?.cast_signed(),
        },
        80 => Instruction::Call { base: input.u16()?, procedure: input.u32()? },
        81 => Instruction::CallAdding {
            base: input.u16()?,
            procedure: input.u16()?,
            from: input.u16()?,
            add: input.u8()?.cast_signed(),
        },
        82 => Instruction::Return { from: input.u16()? },
        83 => Instruction::ReturnIfWholeSmall {
            from: input.u16()?,
            left: input.u16()?,
            right: input.u16()?.cast_signed(),
            compare: input.compare()?,
        },
        84 => Instruction::ReturnAddWhole { left: input.u16()?, right: input.u16()? },
        85 => Instruction::ReturnNothing,
        86 => Instruction::CallDoor { base: input.u16()?, site: input.u32()? },
        87 => Instruction::Fail { failure: input.u16()? },
        88 => Instruction::PassFailureUp,
        89 => Instruction::JumpUnlessFailed { offset: input.u32()?.cast_signed() },
        90 => Instruction::JumpUnlessFailureIs { failure: input.u16()?, offset: input.u32()?.cast_signed() },
        91 => Instruction::ClearFailure,
        92 => Instruction::Crash,
        93 => Instruction::EqualValues { to: input.u16()?, left: input.u16()?, site: input.u16()? },
        94 => Instruction::MakeRange { to: input.u16()?, base: input.u16()?, inclusive: input.flag()? },
        95 => Instruction::PowerWhole { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        96 => Instruction::PowerFloat { to: input.u16()?, left: input.u16()?, right: input.u16()? },
        97 => Instruction::Maths { to: input.u16()?, base: input.u16()?, function: input.u8()? },
        98 => Instruction::LoadNumber { to: input.u16()?, site: input.u16()? },
        99 => Instruction::NumberOperation {
            to: input.u16()?,
            left: input.u16()?,
            right: input.u16()?,
            operation: input.u8()?,
        },
        100 => Instruction::NumberUpdate { target: input.u16()?, right: input.u16()?, operation: input.u8()? },
        101 => Instruction::ConvertNumber { to: input.u16()?, from: input.u16()?, site: input.u16()? },
        102 => Instruction::CheckLimits { register: input.u16()?, site: input.u16()? },
        103 => Instruction::Scale { to: input.u16()?, from: input.u16()?, site: input.u16()? },
        104 => Instruction::CrashIfNone { number: input.u16()? },
        105 => Instruction::JoinPieces { to: input.u16()?, site: input.u32()? },
        106 => Instruction::AppendPieces { to: input.u16()?, site: input.u32()? },
        107 => Instruction::TextToBool { to: input.u16()?, from: input.u16()?, failure: input.u16()? },
        108 => Instruction::TextToWhole { to: input.u16()?, from: input.u16()? },
        109 => Instruction::WholeToFloat { to: input.u16()?, from: input.u16()? },
        110 => Instruction::FloatToWhole { to: input.u16()?, from: input.u16()?, rounding: input.u16()? },
        111 => Instruction::NewList { to: input.u16()?, base: input.u16()?, count: input.u16()? },
        112 => Instruction::NewReferenceList { to: input.u16()?, base: input.u16()?, count: input.u16()? },
        113 => Instruction::RepeatList { to: input.u16()?, value: input.u16()?, count: input.u16()? },
        114 => Instruction::RepeatReferenceList { to: input.u16()?, value: input.u16()?, count: input.u16()? },
        115 => Instruction::ItemAtPosition { to: input.u16()?, list: input.u16()?, index: input.u16()? },
        116 => Instruction::ItemAtOffset { to: input.u16()?, list: input.u16()?, index: input.u16()? },
        117 => Instruction::SetItemAtPosition { list: input.u16()?, index: input.u16()?, value: input.u16()? },
        118 => Instruction::SetItemAtOffset { list: input.u16()?, index: input.u16()?, value: input.u16()? },
        119 => Instruction::AddItem { list: input.u16()?, value: input.u16()? },
        120 => Instruction::InsertItemAtPosition { list: input.u16()?, index: input.u16()?, value: input.u16()? },
        121 => Instruction::InsertItemAtOffset { list: input.u16()?, index: input.u16()?, value: input.u16()? },
        122 => Instruction::CheckRoom { list: input.u16()?, limit: input.u16()? },
        123 => Instruction::RemoveItemAtPosition { list: input.u16()?, index: input.u16()? },
        124 => Instruction::RemoveItemAtOffset { list: input.u16()?, index: input.u16()? },
        125 => Instruction::Length { to: input.u16()?, from: input.u16()? },
        126 => Instruction::NextItem { to: input.u16()?, base: input.u16()?, offset: input.u16()?.cast_signed() },
        127 => Instruction::PrepareRangeTo { base: input.u16()? },
        128 => Instruction::PrepareRangeBefore { base: input.u16()? },
        129 => Instruction::NextInRange { to: input.u16()?, base: input.u16()?, offset: input.u16()?.cast_signed() },
        130 => Instruction::NewDict { to: input.u16()?, shape: input.u16()? },
        131 => Instruction::GetKey { to: input.u16()?, dict: input.u16()?, key: input.u16()? },
        132 => Instruction::SetKey { dict: input.u16()?, key: input.u16()?, value: input.u16()? },
        133 => Instruction::RemoveKey { dict: input.u16()?, key: input.u16()? },
        134 => Instruction::HasKey { to: input.u16()?, dict: input.u16()?, key: input.u16()? },
        135 => Instruction::FindKey { to: input.u16()?, dict: input.u16()?, key: input.u16()? },
        136 => Instruction::DivideFloorByPowerOfTwo { to: input.u16()?, left: input.u16()?, shift: input.u16()? },
        137 => Instruction::RemainderFloorByPowerOfTwo { to: input.u16()?, left: input.u16()?, bits: input.u16()? },
        138 => Instruction::ReleaseText { register: input.u16()? },
        139 => Instruction::GetKeyIfAny { to: input.u16()?, dict: input.u16()?, key: input.u16()? },
        140 => Instruction::GetFoundIfAny { to: input.u16()?, dict: input.u16()?, found: input.u16()? },
        141 => Instruction::GetFound { to: input.u16()?, dict: input.u16()?, found: input.u16()? },
        142 => Instruction::SetFound { dict: input.u16()?, found: input.u16()?, key: input.u16()? },
        143 => Instruction::NextEntry { to: input.u16()?, base: input.u16()?, offset: input.u16()?.cast_signed() },
        144 => Instruction::NewRecord { to: input.u16()?, base: input.u16()?, layout: input.u16()? },
        145 => Instruction::GetField { to: input.u16()?, record: input.u16()?, field: input.u16()? },
        146 => Instruction::SetField { record: input.u16()?, field: input.u16()?, value: input.u16()? },
        147 => Instruction::SetReferenceField { record: input.u16()?, field: input.u16()?, value: input.u16()? },
        148 => Instruction::NewCase { to: input.u16()?, base: input.u16()?, layout: input.u16()? },
        149 => Instruction::UnpackRecord { base: input.u16()?, record: input.u16()? },
        150 => Instruction::PackIntoItemAtPosition { list: input.u16()?, index: input.u16()?, base: input.u16()? },
        151 => Instruction::PackIntoItemAtOffset { list: input.u16()?, index: input.u16()?, base: input.u16()? },
        152 => Instruction::CaseNumber { to: input.u16()?, from: input.u16()? },
        153 => Instruction::GetCaseValue { to: input.u16()?, case: input.u16()?, field: input.u16()? },
        154 => Instruction::Unreachable,
        _ => return Err(FileError::Damaged("an instruction that isn't one")),
    })
}

/// The most registers one instruction names.
pub const MOST_REGISTERS: usize = 3;

/// The most table numbers one instruction holds.
pub const MOST_NUMBERED: usize = 1;

/// The script's tables an instruction picks things out of by number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Table {
    Constant,
    Failure,
    Global,
    Procedure,
    Layout,
}

/// What the verifier checks of an instruction, found in one look at it (D173): the registers it names, in its
/// fields' order, then register 0, which every frame has, for the rest; the numbers it picks things out of the
/// script's tables with; and where it can jump, as an offset from the instruction after it.
#[derive(Clone, Copy, Debug)]
pub struct Facts {
    pub registers: [Register; MOST_REGISTERS],
    pub numbered: [Option<(Table, u32)>; MOST_NUMBERED],
    pub jump: Option<i32>,
}

/// An instruction's [`Facts`].
#[must_use]
pub fn facts(instruction: Instruction) -> Facts {
    match instruction {
        Instruction::Move { to, from, .. } => Facts { registers: [to, from, 0], numbered: [None], jump: None },
        Instruction::LoadSmall { to, .. } => Facts { registers: [to, 0, 0], numbered: [None], jump: None },
        Instruction::LoadConstant { to, constant, .. } => {
            Facts { registers: [to, 0, 0], numbered: [Some((Table::Constant, constant))], jump: None }
        }
        Instruction::LoadText { to, .. } => Facts { registers: [to, 0, 0], numbered: [None], jump: None },
        Instruction::LoadPlainCase { to, .. } => Facts { registers: [to, 0, 0], numbered: [None], jump: None },
        Instruction::GetGlobal { to, global, .. } => {
            Facts { registers: [to, 0, 0], numbered: [Some((Table::Global, u32::from(global)))], jump: None }
        }
        Instruction::SetGlobal { from, global, .. } => {
            Facts { registers: [from, 0, 0], numbered: [Some((Table::Global, u32::from(global)))], jump: None }
        }
        Instruction::Retain { register, .. } => Facts { registers: [register, 0, 0], numbered: [None], jump: None },
        Instruction::Release { register, .. } => Facts { registers: [register, 0, 0], numbered: [None], jump: None },
        Instruction::AddWhole { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::SubtractWhole { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::MultiplyWhole { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::DivideWhole { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::DivideFloor { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::DivideTrunc { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::RemainderFloor { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::RemainderTrunc { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::AddWholeSmall { to, left, .. } => Facts { registers: [to, left, 0], numbered: [None], jump: None },
        Instruction::AddWholeConstant { to, left, constant, .. } => {
            Facts { registers: [to, left, 0], numbered: [Some((Table::Constant, u32::from(constant)))], jump: None }
        }
        Instruction::MultiplyWholeConstant { to, left, constant, .. } => {
            Facts { registers: [to, left, 0], numbered: [Some((Table::Constant, u32::from(constant)))], jump: None }
        }
        Instruction::DivideFloorConstant { to, left, constant, .. } => {
            Facts { registers: [to, left, 0], numbered: [Some((Table::Constant, u32::from(constant)))], jump: None }
        }
        Instruction::RemainderFloorConstant { to, left, constant, .. } => {
            Facts { registers: [to, left, 0], numbered: [Some((Table::Constant, u32::from(constant)))], jump: None }
        }
        Instruction::NegateWhole { to, from, .. } => Facts { registers: [to, from, 0], numbered: [None], jump: None },
        Instruction::AddFloat { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::SubtractFloat { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::MultiplyFloat { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::DivideFloat { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::NegateFloat { to, from, .. } => Facts { registers: [to, from, 0], numbered: [None], jump: None },
        Instruction::SquareRoot { to, from, .. } => Facts { registers: [to, from, 0], numbered: [None], jump: None },
        Instruction::AddFloatConstant { to, left, constant, .. } => {
            Facts { registers: [to, left, 0], numbered: [Some((Table::Constant, u32::from(constant)))], jump: None }
        }
        Instruction::MultiplyFloatConstant { to, left, constant, .. } => {
            Facts { registers: [to, left, 0], numbered: [Some((Table::Constant, u32::from(constant)))], jump: None }
        }
        Instruction::DivideFloatConstant { to, left, constant, .. } => {
            Facts { registers: [to, left, 0], numbered: [Some((Table::Constant, u32::from(constant)))], jump: None }
        }
        Instruction::DivideConstantByFloat { to, right, constant, .. } => {
            Facts { registers: [to, right, 0], numbered: [Some((Table::Constant, u32::from(constant)))], jump: None }
        }
        Instruction::EqualWhole { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::NotEqualWhole { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::LessWhole { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::AtMostWhole { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::UnpackItemAtPosition { base, list, index, .. } => {
            Facts { registers: [base, list, index], numbered: [None], jump: None }
        }
        Instruction::UnpackItemAtOffset { base, list, index, .. } => {
            Facts { registers: [base, list, index], numbered: [None], jump: None }
        }
        Instruction::ItemFieldAtPosition { to, list, index, .. } => {
            Facts { registers: [to, list, index], numbered: [None], jump: None }
        }
        Instruction::ItemFieldAtOffset { to, list, index, .. } => {
            Facts { registers: [to, list, index], numbered: [None], jump: None }
        }
        Instruction::SetItemFieldAtPosition { list, index, value, .. } => {
            Facts { registers: [list, index, value], numbered: [None], jump: None }
        }
        Instruction::SetItemFieldAtOffset { list, index, value, .. } => {
            Facts { registers: [list, index, value], numbered: [None], jump: None }
        }
        Instruction::FitList { to, from, limit, .. } => {
            Facts { registers: [to, from, limit], numbered: [None], jump: None }
        }
        Instruction::Own { register, .. } => Facts { registers: [register, 0, 0], numbered: [None], jump: None },
        Instruction::OwnField { to, record, .. } => Facts { registers: [to, record, 0], numbered: [None], jump: None },
        Instruction::OwnItemAtPosition { to, list, index, .. } => {
            Facts { registers: [to, list, index], numbered: [None], jump: None }
        }
        Instruction::OwnItemAtOffset { to, list, index, .. } => {
            Facts { registers: [to, list, index], numbered: [None], jump: None }
        }
        Instruction::TextLength { to, from, .. } => Facts { registers: [to, from, 0], numbered: [None], jump: None },
        Instruction::GraphemeAtPosition { to, text, index, .. } => {
            Facts { registers: [to, text, index], numbered: [None], jump: None }
        }
        Instruction::GraphemeAtOffset { to, text, index, .. } => {
            Facts { registers: [to, text, index], numbered: [None], jump: None }
        }
        Instruction::Graphemes { to, from, .. } => Facts { registers: [to, from, 0], numbered: [None], jump: None },
        Instruction::SplitText { to, text, by, .. } => {
            Facts { registers: [to, text, by], numbered: [None], jump: None }
        }
        Instruction::SplitTextNoEmpty { to, text, by, .. } => {
            Facts { registers: [to, text, by], numbered: [None], jump: None }
        }
        Instruction::EqualText { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::NotEqualText { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::EqualFloat { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::NotEqualFloat { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::LessFloat { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::AtMostFloat { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::Not { to, from, .. } => Facts { registers: [to, from, 0], numbered: [None], jump: None },
        Instruction::Jump { offset, .. } => Facts { registers: [0, 0, 0], numbered: [None], jump: Some(offset) },
        Instruction::JumpIfTrue { register, offset, .. } => {
            Facts { registers: [register, 0, 0], numbered: [None], jump: Some(offset) }
        }
        Instruction::JumpIfFalse { register, offset, .. } => {
            Facts { registers: [register, 0, 0], numbered: [None], jump: Some(offset) }
        }
        Instruction::JumpIfEqualWhole { left, right, offset, .. } => {
            Facts { registers: [left, right, 0], numbered: [None], jump: Some(i32::from(offset)) }
        }
        Instruction::JumpIfNotEqualWhole { left, right, offset, .. } => {
            Facts { registers: [left, right, 0], numbered: [None], jump: Some(i32::from(offset)) }
        }
        Instruction::JumpIfLessWhole { left, right, offset, .. } => {
            Facts { registers: [left, right, 0], numbered: [None], jump: Some(i32::from(offset)) }
        }
        Instruction::JumpIfAtMostWhole { left, right, offset, .. } => {
            Facts { registers: [left, right, 0], numbered: [None], jump: Some(i32::from(offset)) }
        }
        Instruction::JumpIfEqualWholeSmall { left, offset, .. } => {
            Facts { registers: [left, 0, 0], numbered: [None], jump: Some(i32::from(offset)) }
        }
        Instruction::JumpIfNotEqualWholeSmall { left, offset, .. } => {
            Facts { registers: [left, 0, 0], numbered: [None], jump: Some(i32::from(offset)) }
        }
        Instruction::JumpIfLessWholeSmall { left, offset, .. } => {
            Facts { registers: [left, 0, 0], numbered: [None], jump: Some(i32::from(offset)) }
        }
        Instruction::JumpIfAtMostWholeSmall { left, offset, .. } => {
            Facts { registers: [left, 0, 0], numbered: [None], jump: Some(i32::from(offset)) }
        }
        Instruction::JumpIfGreaterWholeSmall { left, offset, .. } => {
            Facts { registers: [left, 0, 0], numbered: [None], jump: Some(i32::from(offset)) }
        }
        Instruction::JumpIfAtLeastWholeSmall { left, offset, .. } => {
            Facts { registers: [left, 0, 0], numbered: [None], jump: Some(i32::from(offset)) }
        }
        Instruction::JumpIfEqualFloat { left, right, offset, .. } => {
            Facts { registers: [left, right, 0], numbered: [None], jump: Some(i32::from(offset)) }
        }
        Instruction::JumpIfNotEqualFloat { left, right, offset, .. } => {
            Facts { registers: [left, right, 0], numbered: [None], jump: Some(i32::from(offset)) }
        }
        Instruction::JumpIfLessFloat { left, right, offset, .. } => {
            Facts { registers: [left, right, 0], numbered: [None], jump: Some(i32::from(offset)) }
        }
        Instruction::JumpIfAtMostFloat { left, right, offset, .. } => {
            Facts { registers: [left, right, 0], numbered: [None], jump: Some(i32::from(offset)) }
        }
        Instruction::Call { base, procedure, .. } => {
            Facts { registers: [base, 0, 0], numbered: [Some((Table::Procedure, procedure))], jump: None }
        }
        Instruction::CallAdding { base, from, procedure, .. } => {
            Facts { registers: [base, from, 0], numbered: [Some((Table::Procedure, u32::from(procedure)))], jump: None }
        }
        Instruction::Return { from, .. } => Facts { registers: [from, 0, 0], numbered: [None], jump: None },
        Instruction::ReturnIfWholeSmall { from, left, .. } => {
            Facts { registers: [from, left, 0], numbered: [None], jump: None }
        }
        Instruction::ReturnAddWhole { left, right, .. } => {
            Facts { registers: [left, right, 0], numbered: [None], jump: None }
        }
        Instruction::CallDoor { base, .. } => Facts { registers: [base, 0, 0], numbered: [None], jump: None },
        Instruction::Fail { failure, .. } => {
            Facts { registers: [0, 0, 0], numbered: [Some((Table::Failure, u32::from(failure)))], jump: None }
        }
        Instruction::JumpUnlessFailed { offset, .. } => {
            Facts { registers: [0, 0, 0], numbered: [None], jump: Some(offset) }
        }
        Instruction::JumpUnlessFailureIs { failure, offset, .. } => {
            Facts { registers: [0, 0, 0], numbered: [Some((Table::Failure, u32::from(failure)))], jump: Some(offset) }
        }
        Instruction::EqualValues { to, left, .. } => Facts { registers: [to, left, 0], numbered: [None], jump: None },
        Instruction::MakeRange { to, base, .. } => Facts { registers: [to, base, 0], numbered: [None], jump: None },
        Instruction::PowerWhole { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::PowerFloat { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::Maths { to, base, .. } => Facts { registers: [to, base, 0], numbered: [None], jump: None },
        Instruction::LoadNumber { to, .. } => Facts { registers: [to, 0, 0], numbered: [None], jump: None },
        Instruction::NumberOperation { to, left, right, .. } => {
            Facts { registers: [to, left, right], numbered: [None], jump: None }
        }
        Instruction::NumberUpdate { target, right, .. } => {
            Facts { registers: [target, right, 0], numbered: [None], jump: None }
        }
        Instruction::ConvertNumber { to, from, .. } => Facts { registers: [to, from, 0], numbered: [None], jump: None },
        Instruction::CheckLimits { register, .. } => {
            Facts { registers: [register, 0, 0], numbered: [None], jump: None }
        }
        Instruction::Scale { to, from, .. } => Facts { registers: [to, from, 0], numbered: [None], jump: None },
        Instruction::CrashIfNone { number, .. } => Facts { registers: [number, 0, 0], numbered: [None], jump: None },
        Instruction::JoinPieces { to, .. } => Facts { registers: [to, 0, 0], numbered: [None], jump: None },
        Instruction::AppendPieces { to, .. } => Facts { registers: [to, 0, 0], numbered: [None], jump: None },
        Instruction::TextToBool { to, from, failure, .. } => {
            Facts { registers: [to, from, 0], numbered: [Some((Table::Failure, u32::from(failure)))], jump: None }
        }
        Instruction::TextToWhole { to, from, .. } => Facts { registers: [to, from, 0], numbered: [None], jump: None },
        Instruction::WholeToFloat { to, from, .. } => Facts { registers: [to, from, 0], numbered: [None], jump: None },
        Instruction::FloatToWhole { to, from, .. } => Facts { registers: [to, from, 0], numbered: [None], jump: None },
        Instruction::NewList { to, base, .. } => Facts { registers: [to, base, 0], numbered: [None], jump: None },
        Instruction::NewReferenceList { to, base, .. } => {
            Facts { registers: [to, base, 0], numbered: [None], jump: None }
        }
        Instruction::RepeatList { to, value, count, .. } => {
            Facts { registers: [to, value, count], numbered: [None], jump: None }
        }
        Instruction::RepeatReferenceList { to, value, count, .. } => {
            Facts { registers: [to, value, count], numbered: [None], jump: None }
        }
        Instruction::ItemAtPosition { to, list, index, .. } => {
            Facts { registers: [to, list, index], numbered: [None], jump: None }
        }
        Instruction::ItemAtOffset { to, list, index, .. } => {
            Facts { registers: [to, list, index], numbered: [None], jump: None }
        }
        Instruction::SetItemAtPosition { list, index, value, .. } => {
            Facts { registers: [list, index, value], numbered: [None], jump: None }
        }
        Instruction::SetItemAtOffset { list, index, value, .. } => {
            Facts { registers: [list, index, value], numbered: [None], jump: None }
        }
        Instruction::AddItem { list, value, .. } => Facts { registers: [list, value, 0], numbered: [None], jump: None },
        Instruction::InsertItemAtPosition { list, index, value, .. } => {
            Facts { registers: [list, index, value], numbered: [None], jump: None }
        }
        Instruction::InsertItemAtOffset { list, index, value, .. } => {
            Facts { registers: [list, index, value], numbered: [None], jump: None }
        }
        Instruction::CheckRoom { list, limit, .. } => {
            Facts { registers: [list, limit, 0], numbered: [None], jump: None }
        }
        Instruction::RemoveItemAtPosition { list, index, .. } => {
            Facts { registers: [list, index, 0], numbered: [None], jump: None }
        }
        Instruction::RemoveItemAtOffset { list, index, .. } => {
            Facts { registers: [list, index, 0], numbered: [None], jump: None }
        }
        Instruction::Length { to, from, .. } => Facts { registers: [to, from, 0], numbered: [None], jump: None },
        Instruction::NextItem { to, base, offset, .. } => {
            Facts { registers: [to, base, 0], numbered: [None], jump: Some(i32::from(offset)) }
        }
        Instruction::PrepareRangeTo { base, .. } => Facts { registers: [base, 0, 0], numbered: [None], jump: None },
        Instruction::PrepareRangeBefore { base, .. } => Facts { registers: [base, 0, 0], numbered: [None], jump: None },
        Instruction::NextInRange { to, base, offset, .. } => {
            Facts { registers: [to, base, 0], numbered: [None], jump: Some(i32::from(offset)) }
        }
        Instruction::NewDict { to, .. } => Facts { registers: [to, 0, 0], numbered: [None], jump: None },
        Instruction::GetKey { to, dict, key, .. } => Facts { registers: [to, dict, key], numbered: [None], jump: None },
        Instruction::SetKey { dict, key, value, .. } => {
            Facts { registers: [dict, key, value], numbered: [None], jump: None }
        }
        Instruction::RemoveKey { dict, key, .. } => Facts { registers: [dict, key, 0], numbered: [None], jump: None },
        Instruction::HasKey { to, dict, key, .. } => Facts { registers: [to, dict, key], numbered: [None], jump: None },
        Instruction::FindKey { to, dict, key, .. } => {
            Facts { registers: [to, dict, key], numbered: [None], jump: None }
        }
        Instruction::DivideFloorByPowerOfTwo { to, left, .. } => {
            Facts { registers: [to, left, 0], numbered: [None], jump: None }
        }
        Instruction::RemainderFloorByPowerOfTwo { to, left, .. } => {
            Facts { registers: [to, left, 0], numbered: [None], jump: None }
        }
        Instruction::ReleaseText { register, .. } => {
            Facts { registers: [register, 0, 0], numbered: [None], jump: None }
        }
        Instruction::GetKeyIfAny { to, dict, key, .. } => {
            Facts { registers: [to, dict, key], numbered: [None], jump: None }
        }
        Instruction::GetFoundIfAny { to, dict, found, .. } => {
            Facts { registers: [to, dict, found], numbered: [None], jump: None }
        }
        Instruction::GetFound { to, dict, found, .. } => {
            Facts { registers: [to, dict, found], numbered: [None], jump: None }
        }
        Instruction::SetFound { dict, found, key, .. } => {
            Facts { registers: [dict, found, key], numbered: [None], jump: None }
        }
        Instruction::NextEntry { to, base, offset, .. } => {
            Facts { registers: [to, base, 0], numbered: [None], jump: Some(i32::from(offset)) }
        }
        Instruction::NewRecord { to, base, layout, .. } => {
            Facts { registers: [to, base, 0], numbered: [Some((Table::Layout, u32::from(layout)))], jump: None }
        }
        Instruction::GetField { to, record, .. } => Facts { registers: [to, record, 0], numbered: [None], jump: None },
        Instruction::SetField { record, value, .. } => {
            Facts { registers: [record, value, 0], numbered: [None], jump: None }
        }
        Instruction::SetReferenceField { record, value, .. } => {
            Facts { registers: [record, value, 0], numbered: [None], jump: None }
        }
        Instruction::NewCase { to, base, layout, .. } => {
            Facts { registers: [to, base, 0], numbered: [Some((Table::Layout, u32::from(layout)))], jump: None }
        }
        Instruction::UnpackRecord { base, record, .. } => {
            Facts { registers: [base, record, 0], numbered: [None], jump: None }
        }
        Instruction::PackIntoItemAtPosition { list, index, base, .. } => {
            Facts { registers: [list, index, base], numbered: [None], jump: None }
        }
        Instruction::PackIntoItemAtOffset { list, index, base, .. } => {
            Facts { registers: [list, index, base], numbered: [None], jump: None }
        }
        Instruction::CaseNumber { to, from, .. } => Facts { registers: [to, from, 0], numbered: [None], jump: None },
        Instruction::GetCaseValue { to, case, .. } => Facts { registers: [to, case, 0], numbered: [None], jump: None },
        _ => Facts { registers: [0; MOST_REGISTERS], numbered: [None; MOST_NUMBERED], jump: None },
    }
}
