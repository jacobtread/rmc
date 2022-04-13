use std::collections::HashMap;

use gl33::global_loader::{
    glDisableVertexAttribArray, glDrawElements, glEnableVertexAttribArray, glVertexAttribIPointer,
    glVertexAttribPointer,
};
use gl33::*;
use glutin::event::VirtualKeyCode::V;
use glutin::window::CursorIcon::VerticalText;

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
            DataType::UInt | DataType::Int | DataType::Float => 4,
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
            DataType::Int => GL_INT,
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

#[derive(Debug, Clone, Copy)]
struct VertexFormatElement {
    texture_index: u8,
    data_type: DataType,
    type_: ElementType,
    size: GLint,
}

impl VertexFormatElement {
    const POSITION: &'static VertexFormatElement =
        &VertexFormatElement::new(0, DataType::Float, ElementType::Position, 3);
    const COLOR: &'static VertexFormatElement =
        &VertexFormatElement::new(0, DataType::UByte, ElementType::Color, 4);
    const TEXTURE: &'static VertexFormatElement =
        &VertexFormatElement::new(0, DataType::Float, ElementType::UV, 2);
    const OVERLAY: &'static VertexFormatElement =
        &VertexFormatElement::new(1, DataType::Short, ElementType::UV, 2);
    const LIGHT: &'static VertexFormatElement =
        &VertexFormatElement::new(2, DataType::Short, ElementType::UV, 2);
    const NORMAL: &'static VertexFormatElement =
        &VertexFormatElement::new(0, DataType::Byte, ElementType::Normal, 3);
    const PADDING: &'static VertexFormatElement =
        &VertexFormatElement::new(0, DataType::Byte, ElementType::Normal, 1);
}

impl VertexFormatElement {
    const fn new(
        texture_index: u8,
        data_type: DataType,
        type_: ElementType,
        size: GLint,
    ) -> VertexFormatElement {
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
                glVertexAttribPointer(
                    element_index,
                    self.size,
                    data_type.gl(),
                    0,
                    stride,
                    pointer as *const _,
                );
            }
            ElementType::Normal | ElementType::Color => {
                glEnableVertexAttribArray(element_index);
                glVertexAttribPointer(
                    element_index,
                    self.size,
                    data_type.gl(),
                    1,
                    stride,
                    pointer as *const _,
                );
            }
            ElementType::UV => {
                glEnableVertexAttribArray(element_index);
                let gl_type = data_type.gl();
                if gl_type == GL_FLOAT {
                    glVertexAttribPointer(
                        element_index,
                        self.size,
                        gl_type,
                        0,
                        stride,
                        pointer as *const _,
                    );
                } else {
                    glVertexAttribIPointer(
                        element_index,
                        self.size,
                        gl_type,
                        stride,
                        pointer as *const _,
                    );
                }
            }
            ElementType::Padding => {}
        }
    }

    unsafe fn end_drawing(&self, element_index: GLuint) {
        match self.type_ {
            ElementType::Padding => {}
            _ => glDisableVertexAttribArray(element_index),
        }
    }

    fn byte_length(&self) -> usize {
        self.data_type.length() * self.size as usize
    }
}

#[derive(Debug)]
pub struct VertexFormat {
    attr_names: &'static [&'static str],
    elements: &'static [&'static VertexFormatElement],
}

pub struct C(pub u8);

impl VertexFormat {
    pub const A: C = C(0);

    const fn new(
        names: &'static [&'static str],
        values: &'static [&'static VertexFormatElement],
    ) -> VertexFormat {
        VertexFormat {
            attr_names: names,
            elements: values,
        }
    }
}

impl VertexFormat {
    pub const BLIT_SCREEN: &'static VertexFormat = &VertexFormat::new(
        &["Position", "UV", "Color"],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::TEXTURE,
            VertexFormatElement::COLOR,
        ],
    );
    const POSITION_COLOR_TEXTURE_LIGHT_NORMAL: &'static VertexFormat = &VertexFormat::new(
        &["Position", "Color", "UV0", "UV2", "Normal", "Padding"],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::COLOR,
            VertexFormatElement::TEXTURE,
            VertexFormatElement::LIGHT,
            VertexFormatElement::NORMAL,
            VertexFormatElement::PADDING,
        ],
    );
    const POSITION_COLOR_TEXTURE_OVERLAY_LIGHT_NORMAL: &'static VertexFormat = &VertexFormat::new(
        &[
            "Position", "Color", "UV0", "UV1", "UV2", "Normal", "Padding",
        ],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::COLOR,
            VertexFormatElement::TEXTURE,
            VertexFormatElement::OVERLAY,
            VertexFormatElement::LIGHT,
            VertexFormatElement::NORMAL,
            VertexFormatElement::PADDING,
        ],
    );
    const POSITION_TEXTURE_COLOR_LIGHT: &'static VertexFormat = &VertexFormat::new(
        &["Position", "UV0", "Color", "UV2"],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::TEXTURE,
            VertexFormatElement::COLOR,
            VertexFormatElement::LIGHT,
        ],
    );
    pub const POSITION: &'static VertexFormat =
        &VertexFormat::new(&["Position"], &[VertexFormatElement::POSITION]);
    const POSITION_TEXTURE: &'static VertexFormat = &VertexFormat::new(
        &["Position", "UV0"],
        &[VertexFormatElement::POSITION, VertexFormatElement::TEXTURE],
    );
    const POSITION_TEXTURE_COLOR: &'static VertexFormat = &VertexFormat::new(
        &["Position", "UV0", "Color"],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::TEXTURE,
            VertexFormatElement::COLOR,
        ],
    );
    const POSITION_COLOR_TEXTURE: &'static VertexFormat = &VertexFormat::new(
        &["Position", "Color", "UV0"],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::COLOR,
            VertexFormatElement::TEXTURE,
        ],
    );
    const POSITION_COLOR: &'static VertexFormat = &VertexFormat::new(
        &["Position", "Color"],
        &[VertexFormatElement::POSITION, VertexFormatElement::COLOR],
    );
    const POSITION_COLOR_LIGHT: &'static VertexFormat = &VertexFormat::new(
        &["Position", "Color", "UV2"],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::COLOR,
            VertexFormatElement::LIGHT,
        ],
    );
    const POSITION_COLOR_TEXTURE_LIGHT: &'static VertexFormat = &VertexFormat::new(
        &["Position", "Color", "UV0", "UV2"],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::COLOR,
            VertexFormatElement::TEXTURE,
            VertexFormatElement::LIGHT,
        ],
    );
    const POSITION_TEXTURE_LIGHT_COLOR: &'static VertexFormat = &VertexFormat::new(
        &["Position", "UV0", "UV2", "Color"],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::TEXTURE,
            VertexFormatElement::LIGHT,
            VertexFormatElement::COLOR,
        ],
    );
    const POSITION_TEXTURE_COLOR_NORMAL: &'static VertexFormat = &VertexFormat::new(
        &["Position", "UV0", "Color", "Normal", "Padding"],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::TEXTURE,
            VertexFormatElement::COLOR,
            VertexFormatElement::NORMAL,
            VertexFormatElement::PADDING,
        ],
    );
}

struct BufferBuilder {
    buffer: Vec<u8>,
    element_offset: usize,
    format: Option<&'static VertexFormat>,
}

impl BufferBuilder {
    pub fn new(initial_capacity: usize) -> BufferBuilder {
        BufferBuilder {
            buffer: Vec::with_capacity(initial_capacity * 6),
            element_offset: 0,
            format: None,
        }
    }

    pub fn begin() {}

    pub fn put_byte(&mut self, index: usize, value: u8) {
        self.buffer[self.element_offset + index] = value
    }

    pub fn put_short(&mut self, index: usize, value: u16) {
        let v: [u8; 2] = u16::to_be_bytes(value);
        self.buffer[self.element_offset + index] = v[0];
        self.buffer[self.element_offset + index + 1] = v[1];
    }

    pub fn put_float(&mut self, index: usize, value: f32) {
        let v: [u8; 4] = f32::to_be_bytes(value);
        self.buffer[self.element_offset + index] = v[0];
        self.buffer[self.element_offset + index + 1] = v[1];
        self.buffer[self.element_offset + index + 2] = v[2];
        self.buffer[self.element_offset + index + 3] = v[3];
    }
}

macro_rules! constant_refs {
    (
        enum $Name:ident for $StructName:ident {
            $(
                $FieldName:ident = $Expr:expr
            ),* $(,)?
        }
    ) => {
        struct $Name;

        impl $Name {
            $(
                const $FieldName: &'static $StructName = &$Expr;
            )*
        }
    };
}

enum DrawMode {
    Lines,
    LineStrip,
    DebugLines,
    DebugLineStrip,
    Triangles,
    TriangleStrip,
    TriangleFan,
    Quads,
}

impl DrawMode {
    pub fn get_size(&self, vertex_count: usize) -> usize {
        match self {
            DrawMode::Lines | DrawMode::Quads => vertex_count,
            _ => vertex_count / 4 * 6,
        }
    }

    pub fn gl(&self) -> PrimitiveType {
        match self {
            DrawMode::Lines | DrawMode::Triangles => GL_TRIANGLES,
            DrawMode::LineStrip | DrawMode::TriangleStrip | DrawMode::Quads => GL_TRIANGLE_STRIP,
            DrawMode::DebugLines => GL_LINE_STRIP,
            DrawMode::DebugLineStrip => GL_LINES,
            DrawMode::TriangleFan => GL_TRIANGLE_FAN,
        }
    }
}

enum IntType {
    Byte,
    Short,
    Int,
}

impl IntType {}
