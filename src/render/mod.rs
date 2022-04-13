use std::collections::HashMap;

use gl33::*;
use gl33::global_loader::{glDisableVertexAttribArray, glEnableVertexAttribArray, glVertexAttribIPointer, glVertexAttribPointer};
use glutin::event::VirtualKeyCode::V;

use crate::types::{GLint, GLsizei, GLuint};

pub mod shader;
pub mod util;

#[derive(Debug, Copy, Clone)]
enum DataType {
    Float,
    UByte,
    Byte,
    UShort,
    Short,
    UInt,
    Int,
}

impl DataType {
    fn length(&self) -> usize {
        match self {
            DataType::UByte | DataType::Byte => 1,
            DataType::UShort | DataType::Short => 2,
            DataType::UInt | DataType::Int | DataType::Float => 4
        }
    }

    fn name(&self) -> &'static str {
        match self {
            DataType::Float => "float",
            DataType::UByte => "unsigned byte",
            DataType::Byte => "byte",
            DataType::UShort => "unsigned short",
            DataType::Short => "short",
            DataType::UInt => "unsigned int",
            DataType::Int => "int",
        }
    }

    fn gl(&self) -> VertexAttribPointerType {
        match self {
            DataType::Float => GL_FLOAT,
            DataType::UByte => GL_UNSIGNED_BYTE,
            DataType::Byte => GL_BYTE,
            DataType::UShort => GL_UNSIGNED_SHORT,
            DataType::Short => GL_SHORT,
            DataType::UInt => GL_UNSIGNED_INT,
            DataType::Int => GL_INT
        }
    }
}

// enum VertexFormatElement {
//     Position,
//     Color,
//     Texture0,
//     Overlay,
//     Light,
//     Normal,
//     Padding,
// }

#[derive(Debug, Copy, Clone)]
enum ElementType {
    Position,
    Normal,
    Color,
    UV,
    Padding,
}

#[derive(Clone, Copy)]
struct VertexFormatElement {
    texture_index: u8,
    data_type: DataType,
    type_: ElementType,
    size: GLint,
}

impl VertexFormatElement {
    const POSITION: &'static VertexFormatElement = &VertexFormatElement::new(0, DataType::Float, ElementType::Position, 3);
    const COLOR: &'static VertexFormatElement = &VertexFormatElement::new(0, DataType::UByte, ElementType::Color, 4);
    const TEXTURE: &'static VertexFormatElement = &VertexFormatElement::new(0, DataType::Float, ElementType::UV, 2);
    const OVERLAY: &'static VertexFormatElement = &VertexFormatElement::new(1, DataType::Short, ElementType::UV, 2);
    const LIGHT: &'static VertexFormatElement = &VertexFormatElement::new(2, DataType::Short, ElementType::UV, 2);
    const NORMAL: &'static VertexFormatElement = &VertexFormatElement::new(0, DataType::Byte, ElementType::Normal, 3);
    const PADDING: &'static VertexFormatElement = &VertexFormatElement::new(0, DataType::Byte, ElementType::Normal, 1);
}


impl VertexFormatElement {
    const fn new(texture_index: u8, data_type: DataType, type_: ElementType, size: GLint) -> VertexFormatElement {
        VertexFormatElement {
            texture_index,
            data_type,
            type_,
            size,
        }
    }

    unsafe fn start_drawing(&self, element_index: GLuint, stride: GLsizei, pointer: usize) {
        let data_type = &self.data_type;
        match self.type_ {
            ElementType::Position => {
                glEnableVertexAttribArray(element_index);
                glVertexAttribPointer(element_index, self.size, data_type.gl(), 0, stride, pointer as *const _);
            }
            ElementType::Normal | ElementType::Color => {
                glEnableVertexAttribArray(element_index);
                glVertexAttribPointer(element_index, self.size, data_type.gl(), 1, stride, pointer as *const _);
            }
            ElementType::UV => {
                glEnableVertexAttribArray(element_index);
                let gl_type = data_type.gl();
                if gl_type == GL_FLOAT {
                    glVertexAttribPointer(element_index, self.size, gl_type, 0, stride, pointer as *const _);
                } else {
                    glVertexAttribIPointer(element_index, self.size, gl_type, stride, pointer as *const _);
                }
            }
            ElementType::Padding => {}
        }
    }

    unsafe fn end_drawing(&self, element_index: GLuint) {
        match self.type_ {
            ElementType::Padding => {}
            _ => glDisableVertexAttribArray(element_index)
        }
    }

    fn byte_length(&self) -> usize {
        self.data_type.length() * self.size as usize
    }
}

struct VertexFormat<'a> {
    attr_names: &'a [&'a str],
    elements: &'a [&'static VertexFormatElement],
}

impl<'a> VertexFormat<'a> {
    const fn new(names: &'a [&'a str], values: &'a [&'static VertexFormatElement]) -> VertexFormat<'a> {
        VertexFormat {
            attr_names: names,
            elements: values,
        }
    }
}

struct VertexFormats;

impl VertexFormat {
    const BLIT_SCREEN: &'static VertexFormat<'static> = &VertexFormat::new(&["Position", "UV", "Color"], &[VertexFormatElement::POSITION, VertexFormatElement::TEXTURE, VertexFormatElement::COLOR]);
    const POSITION_COLOR_TEXTURE_LIGHT_NORMAL: &'static VertexFormat<'static> = &VertexFormat::new(&["Position", "Color", "UV0", "UV2", "Normal", "Padding"], &[VertexFormatElement::POSITION, VertexFormatElement::COLOR, VertexFormatElement::TEXTURE, VertexFormatElement::LIGHT, VertexFormatElement::NORMAL, VertexFormatElement::PADDING]);
    const POSITION_COLOR_TEXTURE_OVERLAY_LIGHT_NORMAL: &'static VertexFormat<'static> = &VertexFormat::new(&["Position", "Color", "UV0", "UV1", "UV2", "Normal", "Padding"], &[VertexFormatElement::POSITION, VertexFormatElement::COLOR, VertexFormatElement::TEXTURE, VertexFormatElement::OVERLAY, VertexFormatElement::LIGHT, VertexFormatElement::NORMAL, VertexFormatElement::PADDING]);
    const POSITION_TEXTURE_COLOR_LIGHT: &'static VertexFormat<'static> = &VertexFormat::new(&["Position", "UV0", "Color", "UV2"], &[VertexFormatElement::POSITION, VertexFormatElement::TEXTURE, VertexFormatElement::COLOR, VertexFormatElement::LIGHT]);
    const POSITION: &'static VertexFormat<'static> = &VertexFormat::new(&["Position"], &[VertexFormatElement::POSITION]);
    const POSITION_TEXTURE: &'static VertexFormat<'static> = &VertexFormat::new(&["Position", "UV0"], &[VertexFormatElement::POSITION, VertexFormatElement::TEXTURE]);
    const POSITION_TEXTURE_COLOR: &'static VertexFormat<'static> = &VertexFormat::new(&["Position", "UV0", "Color"], &[VertexFormatElement::POSITION, VertexFormatElement::TEXTURE, VertexFormatElement::COLOR]);
    const POSITION_COLOR_TEXTURE: &'static VertexFormat<'static> = &VertexFormat::new(&["Position", "Color", "UV0"], &[VertexFormatElement::POSITION, VertexFormatElement::COLOR, VertexFormatElement::TEXTURE]);
    const POSITION_COLOR: &'static VertexFormat<'static> = &VertexFormat::new(&["Position", "Color"], &[VertexFormatElement::POSITION, VertexFormatElement::COLOR]);
    const POSITION_COLOR_LIGHT: &'static VertexFormat<'static> = &VertexFormat::new(&["Position", "Color", "UV2"], &[VertexFormatElement::POSITION, VertexFormatElement::COLOR, VertexFormatElement::LIGHT]);
    const POSITION_COLOR_TEXTURE_LIGHT: &'static VertexFormat<'static> = &VertexFormat::new(&["Position", "Color", "UV0", "UV2"], &[VertexFormatElement::POSITION, VertexFormatElement::COLOR, VertexFormatElement::TEXTURE, VertexFormatElement::LIGHT]);
    const POSITION_TEXTURE_LIGHT_COLOR: &'static VertexFormat<'static> = &VertexFormat::new(&["Position", "UV0", "UV2", "Color"], &[VertexFormatElement::POSITION, VertexFormatElement::TEXTURE, VertexFormatElement::LIGHT, VertexFormatElement::COLOR]);
    const POSITION_TEXTURE_COLOR_NORMAL: &'static VertexFormat<'static> = &VertexFormat::new(&["Position", "UV0", "Color", "Normal", "Padding", ], &[VertexFormatElement::POSITION, VertexFormatElement::TEXTURE,VertexFormatElement::COLOR, VertexFormatElement::NORMAL, VertexFormatElement::PADDING]);
}

// struct BufferBuilder<'a> {
//     buffer: &'a mut [u8],
//     element_offset: usize,
//     format: Option<VertexFormat>,
// }
//
// impl BufferBuilder {
//     pub fn new() -> BufferBuilder {
//         BufferBuilder {
//             buffer: &[0u8; 1536],
//             element_offset: 0,
//         }
//     }
// }
