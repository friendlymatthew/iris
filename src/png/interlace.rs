/*
    pass 1: 8x, col: 8x
    pass 2: 8x + 4, col: 8x
    pass 3: 4x, col: 8x + 4
    pass 4: 4x + 2, col: 4x
    pass 5: 2x, col: 2 + 4x
    pass 6: 1 + 2x, col: 2x
    pass 7: x, col: 1 + 2x

[1, 6, 4, 6, 2, 6, 4, 6],[1, 6, 4, 6, 2, 6, 4, 6],
[7, 7, 7, 7, 7, 7, 7, 7],[7, 7, 7, 7, 7, 7, 7, 7],
[5, 6, 5, 6, 5, 6, 5, 6],[5, 6, 5, 6, 5, 6, 5, 6],
[7, 7, 7, 7, 7, 7, 7, 7],[7, 7, 7, 7, 7, 7, 7, 7],
[3, 6, 4, 6, 3, 6, 4, 6],[3, 6, 4, 6, 3, 6, 4, 6],
[7, 7, 7, 7, 7, 7, 7, 7],[7, 7, 7, 7, 7, 7, 7, 7],
[5, 6, 5, 6, 5, 6, 5, 6],[5, 6, 5, 6, 5, 6, 5, 6],
[7, 7, 7, 7, 7, 7, 7, 7],[7, 7, 7, 7, 7, 7, 7, 7],
[1, 6, 4, 6, 2, 6, 4, 6],[1, ...
[7, 7, 7, 7, 7, 7, 7, 7],
[5, 6, 5, 6, 5, 6, 5, 6],
[7, 7, 7, 7, 7, 7, 7, 7],
[3, 6, 4, 6, 3, 6, 4, 6],
[7, 7, 7, 7, 7, 7, 7, 7],
[5, 6, 5, 6, 5, 6, 5, 6],
[7, 7, 7, 7, 7, 7, 7, 7],
*/

pub struct Pass {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) compute_x: Box<dyn Fn(usize) -> usize>,
    pub(crate) compute_y: Box<dyn Fn(usize) -> usize>,
}

pub fn compute_pass_counts(width: u32, height: u32) -> [Pass; 7] {
    let width = width as usize;
    let height = height as usize;

    [
        Pass {
            width: width.div_ceil(8),
            height: height.div_ceil(8),
            compute_x: Box::new(|x| 8 * x),
            compute_y: Box::new(|y| 8 * y),
        },
        Pass {
            width: (width - 4).div_ceil(8),
            height: height.div_ceil(8),
            compute_x: Box::new(|x| 8 * x + 4),
            compute_y: Box::new(|y| 8 * y),
        },
        Pass {
            width: width.div_ceil(4),
            height: (height - 4).div_ceil(8),
            compute_x: Box::new(|x| 4 * x),
            compute_y: Box::new(|y| 8 * y + 4),
        },
        Pass {
            width: (width - 2).div_ceil(4),
            height: height.div_ceil(4),
            compute_x: Box::new(|x| 4 * x + 2),
            compute_y: Box::new(|y| 4 * y),
        },
        Pass {
            width: width.div_ceil(2),
            height: (height - 2).div_ceil(4),
            compute_x: Box::new(|x| 2 * x),
            compute_y: Box::new(|y| 4 * y + 2),
        },
        Pass {
            width: (width - 1).div_ceil(2),
            height: height.div_ceil(2),
            compute_x: Box::new(|x| 2 * x + 1),
            compute_y: Box::new(|y| 2 * y),
        },
        Pass {
            width,
            height: (height - 1).div_ceil(2),
            compute_x: Box::new(|x| x),
            compute_y: Box::new(|y| 2 * y + 1),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass_count_for_8x8() {
        let pass_cts = compute_pass_counts(8, 8);

        assert_eq!(
            pass_cts
                .into_iter()
                .map(|p| p.width * p.height)
                .collect::<Vec<_>>(),
            vec![1 << 0, 1 << 0, 1 << 1, 1 << 2, 1 << 3, 1 << 4, 1 << 5,]
        )
    }

    #[test]
    fn pass_count_for_9x9() {
        let pass_cts = compute_pass_counts(9, 9);
        assert_eq!(
            pass_cts
                .into_iter()
                .map(|p| p.width * p.height)
                .collect::<Vec<_>>(),
            vec![4, 2, 3, 6, 10, 20, 36]
        )
    }

    #[test]
    fn pass_count_for_9x10() {
        let pass_cts = compute_pass_counts(9, 10);
        assert_eq!(
            pass_cts
                .into_iter()
                .map(|p| p.width * p.height)
                .collect::<Vec<_>>(),
            vec![4, 2, 3, 6, 10, 20, 45]
        )
    }
}
