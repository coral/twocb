pub fn add(
    op1: Vec<vecmath::Vector4<f64>>,
    mut op2: Vec<vecmath::Vector4<f64>>,
) -> Vec<vecmath::Vector4<f64>> {
    for (i, _v) in op1.iter().enumerate() {
        // op2[i] = vecmath::vec4_add(op1[i], op2[i]);
        // op2[i][0] = op2[i][0].clamp(0.0, 1.0);
        // op2[i][1] = op2[i][1].clamp(0.0, 1.0);
        // op2[i][2] = op2[i][2].clamp(0.0, 1.0);

        op2[i][0] = (op1[i][0] + (op2[i][0] * op2[i][3])).clamp(0.0, 0.1);
        op2[i][1] = (op1[i][1] + (op2[i][1] * op2[i][3])).clamp(0.0, 0.1);
        op2[i][2] = (op1[i][2] + (op2[i][2] * op2[i][3])).clamp(0.0, 0.1);
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
    fn blending_add() {
        let op1 = vec![[1.0, 1.0, 1.0, 0.0]];
        let op2 = vec![[0.5, 0.5, 0.5, 0.0]];
        let m = add(op1, op2);
        assert!(m[0][0] == 1.0);
        assert!(m[0][1] == 1.0);
        assert!(m[0][2] == 1.0);

        let op1 = vec![[0.0, 0.2, 0.5, 1.0]];
        let op2 = vec![[0.0, 0.5, 0.5, 1.0]];
        let m = add(op1, op2);
        assert!(m[0][0] == 0.0);
        assert!(m[0][1] == 0.7);
        assert!(m[0][2] == 1.0);
    }
}
