use super::Buffer;
use std::rc::Rc;

#[derive(Default)]
pub struct Frame {
    pub(crate) usage_count: u64,
    pub(crate) buffer: Rc<Buffer>,
}

impl Frame {
    pub(crate) fn use_buffer(&mut self) -> Rc<Buffer> {
        self.usage_count += 1;
        self.buffer.clone()
    }

    pub(crate) fn has_reference(&mut self) -> bool {
        Rc::get_mut(&mut self.buffer).is_none()
    }
}

#[cfg(test)]
mod frame_test {
    use super::*;

    mod use_buffer {
        use super::*;

        #[test]
        fn カウントを1だけ加算しバッファのクローンを返すこと() {
            // Arrange
            let mut frame = Frame {
                usage_count: 0,
                buffer: Rc::new(Buffer::default()),
            };

            // Act
            let buffer = frame.use_buffer();

            // Assert
            assert_eq!(frame.usage_count, 1);
            assert_eq!(buffer, frame.buffer);
        }
    }

    mod has_reference {
        use super::*;

        #[test]
        fn 強い参照を持つ場合trueとなること() {
            // Arrange
            let mut frame = Frame {
                usage_count: 0,
                buffer: Rc::new(Buffer::default()),
            };
            let _clone = Rc::clone(&frame.buffer);

            // Act
            let actual = frame.has_reference();

            // Assert
            assert_eq!(actual, true);
        }

        #[test]
        fn 参照を持たない場合falseとなること() {
            // Arrange
            let mut frame = Frame {
                usage_count: 0,
                buffer: Rc::new(Buffer::default()),
            };

            // Act
            let actual = frame.has_reference();

            // Assert
            assert_eq!(actual, false);
        }
    }
}
