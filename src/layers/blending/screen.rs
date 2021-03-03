pub fn screen(
    op1: Vec<vecmath::Vector4<f64>>,
    mut op2: Vec<vecmath::Vector4<f64>>,
) -> Vec<vecmath::Vector4<f64>> {
    for (i, v) in op1.iter().enumerate() {
        op2[i][0] = (1.0 - (1.0 - op1[i][0]) * (1.0 - op2[i][0])).clamp(0.0, 1.0);
        op2[i][1] = (1.0 - (1.0 - op1[i][1]) * (1.0 - op2[i][1])).clamp(0.0, 1.0);
        op2[i][2] = (1.0 - (1.0 - op1[i][2]) * (1.0 - op2[i][2])).clamp(0.0, 1.0);
    }

    return op2;
}

////////////
/// TESTS
/////////////

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blending_screen() {
        let op1 = vec![[0.3, 0.2, 0.4, 0.0]];
        let op2 = vec![[0.3, 0.5, 0.5, 0.0]];
        let m = screen(op1, op2);
        assert!(m[0][0] == 0.51);
        assert!(m[0][1] == 0.6);
        assert!(m[0][2] == 0.7);

        let op1 = vec![[1.0, -0.2, 0.4, 0.0]];
        let op2 = vec![[1.0, 2.5, 0.5, 0.0]];
        let m = screen(op1, op2);
        assert!(m[0][0] == 1.0);
        assert!(m[0][1] == 1.0);
        assert!(m[0][2] == 0.7);
    }
}
