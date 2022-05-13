use ark_ff::{Field, CubicExtParameters};
use elusiv_interpreter::elusiv_computation;
use ark_bn254::{ Fq, Fq2, Fq6, Fq12, Fq12Parameters, G1Affine, G2Affine, Fq6Parameters, Parameters };
use ark_ff::fields::models::{ QuadExtParameters, fp12_2over3over2::Fp12ParamsWrapper, fp6_3over2::Fp6ParamsWrapper };
use ark_ff::{ One, biginteger::BigInteger256, field_new };
use ark_ec::models::bn::BnParameters;

// https://github.com/arkworks-rs/curves/blob/1551d6d76ce5abf6e7925e53b0ea1af7dbc421c3/bn254/src/curves/mod.rs#L21
const ATE_LOOP_COUNT: [i8; 65] = [0,0,0,1,0,1,0,-1,0,0,1,-1,0,0,1,0,0,1,1,0,-1,0,0,1,0,-1,0,0,0,0,1,1,1,0,0,-1,0,0,1,0,0,0,0,0,-1,0,0,1,1,0,0,-1,0,0,0,1,1,0,-1,0,0,1,0,1,1];

// We combine the miller loop and the coefficient generation (conversion for B from G2Affine to G2Prepared)
// - miller loop ref: https://github.com/arkworks-rs/algebra/blob/6ea310ef09f8b7510ce947490919ea6229bbecd6/ec/src/models/bn/mod.rs#L99
// - generation ref: https://github.com/arkworks-rs/algebra/blob/6ea310ef09f8b7510ce947490919ea6229bbecd6/ec/src/models/bn/g2.rs#L68
// - implementation:
// - the miller loop receives an iterator over 3 elements (https://github.com/arkworks-rs/groth16/blob/765817f77a6e14964c6f264d565b18676b11bd59/src/verifier.rs#L41)

// - for B we need to generate the coeffs (all other coeffs already are generated befor compilation)
// - so we have a var r = (x: rbx, y: rby, z: rbz)

pub fn new_g2_hom_projective(x: Fq2, y: Fq2, z: Fq2) -> G2HomProjective {
    G2HomProjective { x, y, z }
}

/*
mul_by_034(f, ( mul_by_fp(c0, a.y), mul_by_fp(c1, a.x), c2));
mul_by_034(f, ( mul_by_fp(VKey::gamma_g2_neg_pc(coeff_ic).0, p_inputs.y), mul_by_fp(VKey::gamma_g2_neg_pc(coeff_ic).1, p_inputs.x), VKey::gamma_g2_neg_pc(coeff_ic).2,));
mul_by_034(f, ( mul_by_fp(VKey::delta_g2_neg_pc(coeff_ic).0, c.y), mul_by_fp(VKey::delta_g2_neg_pc(coeff_ic).1, c.x), VKey::delta_g2_neg_pc(coeff_ic).2,));
*/

#[derive(Debug)]
// Homogenous projective coordinates form
pub struct G2HomProjective {
    pub x: Fq2,
    pub y: Fq2,
    pub z: Fq2,
}

/// Inverse of 2 (in q)
/// - Calculated using: Fq::one().double().inverse().unwrap()
pub const TWO_INV: Fq = Fq::new(BigInteger256::new([9781510331150239090, 15059239858463337189, 10331104244869713732, 2249375503248834476]));

/// https://docs.rs/ark-bn254/0.3.0/src/ark_bn254/curves/g2.rs.html#19
/// COEFF_B = 3/(u+9) = (19485874751759354771024239261021720505790618469301721065564631296452457478373, 266929791119991161246907387137283842545076965332900288569378510910307636690)
const COEFF_B: Fq2 = field_new!(Fq2,
    field_new!(Fq, "19485874751759354771024239261021720505790618469301721065564631296452457478373"),
    field_new!(Fq, "266929791119991161246907387137283842545076965332900288569378510910307636690"),
);

pub struct CoefficientsResult {
    new_r: G2HomProjective,
    coeffs: (Fq2, Fq2, Fq2),
}

// Doubling step
// https://github.com/arkworks-rs/algebra/blob/6ea310ef09f8b7510ce947490919ea6229bbecd6/ec/src/models/bn/g2.rs#L139
elusiv_computation!(
    doubling_step (r: G2HomProjective) -> Fq12,
    {
        {
            let mut a: Fq2 = r.x * r.y;
            a = mul_by_fp(a, TWO_INV);
            let b: Fq2 = square_fq2(r.y);
            let c: Fq2 = square_fq2(r.z);
            let e: Fq2 = COEFF_B * (double(c) + c);
            let f: Fq2 = double(e) + e;
            let mut g: Fq2 = b + f;
            g = mul_by_fp(g, TWO_INV);
            let h: Fq2 = square_fq2(r.y + r.z) - (b + c);
            let e_square: Fq2 = square_fq2(e);
        }
        {
            let rx: Fq2 = a * (b - f);
            let ry: Fq2 = square_fq2(g) - (double(e_square) + e_square);
            let rz: Fq2 = b * h;
        }
        {
            let i: Fq2 = e - b;
            let j: Fq2 = square_fq2(r.x);
            let res: CoefficientsResult = new_coeff_result(rx, ry, rz, neg_fq2(h), double(j) + j, i);
            return res;
        }
    }
);

pub fn neg_fq2(v: Fq2) -> Fq2 { -v }
pub fn double(v: Fq2) -> Fq2 { v.double() }
pub fn new_coeff_result(x: Fq2, y: Fq2, z: Fq2, c0: Fq2, c1: Fq2, c2: Fq2) -> CoefficientsResult {
    CoefficientsResult {
        new_r: G2HomProjective { x, y, z },
        coeffs: (c0, c1, c2),
    }
}

// Addition step
// https://github.com/arkworks-rs/algebra/blob/6ea310ef09f8b7510ce947490919ea6229bbecd6/ec/src/models/bn/g2.rs#L168
elusiv_computation!(
    addition_step (r: G2HomProjective, q: G2Affine),
    {
        {
            let theta: Fq2 = r.y - (q.y * r.z);
            let lambda: Fq2 = r.x - (q.x * r.z);
            let c: Fq2 = square_fq2(theta);
            let d: Fq2 = square_fq2(lambda);
            let e: Fq2 = lambda * d;
            let f: Fq2 = r.z * c;
            let g: Fq2 = r.x * d;
            let h: Fq2 = e + f - double(g);
            let rx: Fq2 = lambda * h;
            let ry: Fq2 = theta * (g - h) - (e * r.y);
            let rz: Fq2 = r.z * e;
            let j: Fq2 = theta * q.x - (lambda * q.y);

            let res: CoefficientsResult = new_coeff_result(rx, ry, rz, lambda, neg_fq2(theta), j);
            return res;
        }
    }
);

const TWIST_MUL_BY_Q_X: Fq2 = Parameters::TWIST_MUL_BY_Q_X;
const TWIST_MUL_BY_Q_Y: Fq2 = Parameters::TWIST_MUL_BY_Q_Y;

// Mul by characteristics
// https://github.com/arkworks-rs/algebra/blob/6ea310ef09f8b7510ce947490919ea6229bbecd6/ec/src/models/bn/g2.rs#L127
elusiv_computation!(
    mul_by_characteristics (r: G2Affine),
    {
        {
            let mut x: Fq2 = frobenius_map_fq2(r.x, 1);
            x = x * TWIST_MUL_BY_Q_X;
        }
        {
            let mut y: Fq2 = frobenius_map_fq2(r.y, 1);
            y = y * TWIST_MUL_BY_Q_Y;
            let res: G2Affine = new_g2affine(x, y, r.infinity);
        }
    }
);

pub fn new_g2affine(x: Fq2, y: Fq2, infinity: bool) -> G2Affine {
    G2Affine::new(x, y, infinity)
}

pub fn frobenius_map_fq2(f: Fq2, u: usize) -> Fq2 {
    let mut k = f.clone();
    k.frobenius_map(u);
    k
}

// Coefficients ell

// Line function evaluation at point p
// - since miller loop calls ell_round on 1. A, a prepared input, C we combine all three 
// - reference implementation: https://github.com/arkworks-rs/algebra/blob/6ea310ef09f8b7510ce947490919ea6229bbecd6/ec/src/models/bn/mod.rs#L59
elusiv_computation!(
    ell (ram_fq6: &mut RAM<Fq6>, coeffs: (Fq2, Fq2, Fq2), p: G1Affine, f: Fq12),
    {
        {
            let c0: Fq2 = mul_by_fp(coeffs.0, p.y);
            let c1: Fq2 = mul_by_fp(coeffs.1, p.x);
        }
        { partial let res: Fq12 = mul_by_034(ram_fq6, c0, c1, coeffs.2, f); }
        { return res; }
    }
);

// f.mul_by_034(c0, c1, coeffs.2); (with: self -> f; c0 -> c0; d0 -> c1; d1 -> coeffs.2)
// https://github.com/arkworks-rs/r1cs-std/blob/b7874406ec614748608b1739b1578092a8c97fb8/src/fields/fp12.rs#L43
elusiv_computation!(
    mul_by_034 (c0: Fq2, d0: Fq2, d1: Fq2, f: Fq12),
    {
        { let a: Fq6 = new_fq6(f.c0.c0 * c0, f.c0.c1 * c0, f.c0.c2 * c0); }
        { let b: Fq6 = mul_fq6_by_c0_c1_0(f.c1, d0, d1); }
        { let e: Fq6 = mul_fq6_by_c0_c1_0(f.c0 + f.c1, c0 + d0, d1); }
        {
            let res: Fq12 = new_fq12(mul_base_field_by_nonresidue(b) + a, e - (a + b));
            return res;
        }
    }
);

// https://github.com/arkworks-rs/algebra/blob/4dd6c3446e8ab22a2ba13505a645ea7b3a69f493/ff/src/fields/models/quadratic_extension.rs#L87
// https://github.com/arkworks-rs/algebra/blob/4dd6c3446e8ab22a2ba13505a645ea7b3a69f493/ff/src/fields/models/quadratic_extension.rs#L56
fn sub_and_mul_base_field_by_nonresidue(x: Fq6, y: Fq6) -> Fq6 {
    x - mul_base_field_by_nonresidue(y)
}

pub fn mul_base_field_by_nonresidue(v: Fq6) -> Fq6 {
    Fp12ParamsWrapper::<Fq12Parameters>::mul_base_field_by_nonresidue(&v)
}

// https://github.com/arkworks-rs/r1cs-std/blob/b7874406ec614748608b1739b1578092a8c97fb8/src/fields/fp6_3over2.rs#L53
pub fn mul_fq6_by_c0_c1_0(f: Fq6, c0: Fq2, c1: Fq2) -> Fq6 {
    let v0: Fq2 = f.c0 * c0;
    let v1: Fq2 = f.c1 * c1;

    let a1_plus_a2: Fq2 = f.c1 + f.c2;
    let a0_plus_a1: Fq2 = f.c0 + f.c1;
    let a0_plus_a2: Fq2 = f.c0 + f.c2;

    let b1_plus_b2: Fq2 = c1.clone();
    let b0_plus_b1: Fq2 = c0 + c1;
    let b0_plus_b2: Fq2 = c0.clone();

    Fq6::new(
        (a1_plus_a2 * b1_plus_b2 - &v1) * Fp6ParamsWrapper::<Fq6Parameters>::NONRESIDUE + &v0,
        a0_plus_a1 * &b0_plus_b1 - &v0 - &v1,
        a0_plus_a2 * &b0_plus_b2 - &v0 + &v1,
    )
}

pub fn new_fq12(c0: Fq6, c1: Fq6) -> Fq12 { Fq12::new(c0, c1) }
pub fn new_fq6(c0: Fq2, c1: Fq2, c2: Fq2) -> Fq6 { Fq6::new(c0, c1, c2) }

pub fn mul_by_fp(v: Fq2, fp: Fq) -> Fq2 {
    let mut v = v;
    v.mul_assign_by_fp(&fp);
    v
}

// Final exponentiation
// - reference implementation: https://github.com/arkworks-rs/algebra/blob/6ea310ef09f8b7510ce947490919ea6229bbecd6/ec/src/models/bn/mod.rs#L153
elusiv_computation!(
    final_exponentiation (ram_fq6: &mut RAM<Fq6>, f: Fq12),
    {
        { let f1: Fq12 = conjugate(f); }
        { partial let f2: Fq12 = inverse_fq12(ram_fq6, f); }
        {
            let r: Fq12 = f1 * f2;
            f2 = r;
        }
        {
            r = frobenius_map(r, 2);
            r = r * f2;
        }
        { partial let y0: Fq12 = exp_by_neg_x(ram_fq12, r); }
        {
            let y1: Fq12 = cyclotomic_square(y0);
            let y2: Fq12 = cyclotomic_square(y1);
            let y3: Fq12 = y2 * y1;
        }
        { partial let y4: Fq12 = exp_by_neg_x(ram_fq12, y3); }
        { let y5: Fq12 = cyclotomic_square(y4); }
        { partial let y6: Fq12 = exp_by_neg_x(ram_fq12, y5); }
        {
            y3 = conjugate(y3);
            y6 = conjugate(y6);
        }
        {
            let y7: Fq12 = y6 * y4;
            let y8: Fq12 = y7 * y3;
            let y9: Fq12 = y8 * y1;
            let y10: Fq12 = y8 * y4;
        }
        {
            let y11: Fq12 = y10 * r;
            let mut y12: Fq12 = y9;
            y12 = frobenius_map(y12, 1);
        }
        {
            let y13: Fq12 = y12 * y11;
            y8 = frobenius_map(y8, 2);
        }
        {
            let y14: Fq12 = y8 * y13;
            r = conjugate(r);
        }
        {
            let mut y15: Fq12 = r * y9;
            y15 = frobenius_map(y15, 3);
            let y16: Fq12 = y15 * y14;

            return y16;
        }
    }
);

// https://github.com/arkworks-rs/algebra/blob/4dd6c3446e8ab22a2ba13505a645ea7b3a69f493/ff/src/fields/models/quadratic_extension.rs#L366
// Guide to Pairing-based Cryptography, Algorithm 5.19.
elusiv_computation!(
    inverse_fq12 (f: Fq12),
    {
        { let v1: Fq6 = square_fq6(f.c1); }
        { let v2: Fq6 = square_fq6(f.c0); }
        { let mut v0: Fq6 = sub_and_mul_base_field_by_nonresidue(v2, v1); }
        { unwrap let v3: Fq6 = inverse_fq6(v0); }
        {
            let res: Fq12 = new_fq12(f.c0 * v3, neg_fq6(f.c1 * v3));
            return res;
        }
    }
);

// Using exp_by_neg_x and cyclotomic_exp
// https://github.com/arkworks-rs/algebra/blob/6ea310ef09f8b7510ce947490919ea6229bbecd6/ec/src/models/bn/mod.rs#L78
// https://github.com/arkworks-rs/algebra/blob/6ea310ef09f8b7510ce947490919ea6229bbecd6/ff/src/fields/models/fp12_2over3over2.rs#L56
elusiv_computation!(
    exp_by_neg_x (fe: Fq12),
    {
        {
            let fe_inverse: Fq12 = conjugate(fe);
            let res: Fq12 = one();
        }

        // Non-adjacent window form of exponent Parameters::X (u64: 4965661367192848881)
        // NAF computed using: https://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.394.3037&rep=rep1&type=pdf Page 98
        // - but removed the last zero value, since it has no effect
        // - and then inverted the array
        { for value in [1,0,0,0,1,0,1,0,0,2,0,1,0,1,0,2,0,0,1,0,1,0,2,0,2,0,2,0,1,0,0,0,1,0,0,1,0,1,0,1,0,2,0,1,0,0,1,0,0,0,0,1,0,1,0,0,0,0,2,0,0,0,1]
            {
                if (larger_than_zero(round)) {
                    res = cyclotomic_square(res);
                };

                if (is_non_zero(value)) {
                    if (is_one(value)) {
                        res = res * fe;
                    } else { // value == 2
                        res = res * fe_inverse;
                    };
                };
            }
        }
        {
            let a: Fq12 = conjugate(res);
            return a;
        }
    }
);
pub fn is_non_zero(v: u8) -> bool { v != 0 }
pub fn larger_than_zero(v: usize) -> bool { v > 0 }
pub fn is_one(v: u8) -> bool { v == 1 }
pub fn one() -> Fq12 { Fq12::one() }

pub fn neg_fq6(v: Fq6) -> Fq6 { -v }
pub fn square_fq6(c: Fq6) -> Fq6 { c.square() }
pub fn square_fq2(c: Fq2) -> Fq2 { c.square() }
pub fn inverse_fq6(v: Fq6) -> Option<Fq6> { v.inverse() }

pub fn conjugate(f: Fq12) -> Fq12 {
    let mut k = f.clone();
    k.conjugate();
    k
}
pub fn frobenius_map(f: Fq12, u: usize) -> Fq12 {
    let mut k = f.clone();
    k.frobenius_map(u);
    k
}
pub fn cyclotomic_square(f: Fq12) -> Fq12 {
    f.cyclotomic_square()
}

pub struct RAM<N: Clone> {
    data: Vec<Option<N>>,
    frame: usize,
}

impl<N: Clone> RAM<N> {
    pub fn new(size: usize) -> Self {
        let mut data = vec![];
        for _ in 0..size { data.push(None); }
        RAM { data, frame: 0 }
    }

    pub fn write(&mut self, value: N, index: usize) {
        self.data[self.frame + index] = Some(value);
    }

    pub fn read(&self, index: usize) -> N {
        self.data[self.frame + index].clone().unwrap()
    }

    pub fn free(&mut self, index: usize) {
        self.data[self.frame + index] = None;
    }

    pub fn inc_frame(&mut self, frame: usize) {
        self.frame += frame;
    }

    pub fn dec_frame(&mut self, frame: usize) {
        self.frame -= frame;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use ark_bn254::{ Bn254, Fq, Fq2, Parameters };
    use ark_ec::PairingEngine;
    use ark_ec::models::bn::BnParameters;

    fn f() -> Fq12 {
        let f = Fq6::new(
            Fq2::new(
                Fq::from_str("20925091368075991963132407952916453596237117852799702412141988931506241672722").unwrap(),
                Fq::from_str("18684276579894497974780190092329868933855710870485375969907530111657029892231").unwrap(),
            ),
            Fq2::new(
                Fq::from_str("5932690455294482368858352783906317764044134926538780366070347507990829997699").unwrap(),
                Fq::from_str("18684276579894497974780190092329868933855710870485375969907530111657029892231").unwrap(),
            ),
            Fq2::new(
                Fq::from_str("18684276579894497974780190092329868933855710870485375969907530111657029892231").unwrap(),
                Fq::from_str("19526707366532583397322534596786476145393586591811230548888354920504818678603").unwrap(),
            ),
        );
        Fq12::new(f, f)
    }

    fn coeffs() -> (Fq2, Fq2, Fq2) { (f().c0.c0, f().c1.c0, f().c0.c2) }
    fn g1_affine() -> G1Affine { G1Affine::new(f().c0.c0.c0, f().c0.c0.c1, false) }

    #[test]
    fn test_ell() {
        let mut ram_fq12: RAM<Fq12> = RAM::new(20);
        let mut ram_fq6: RAM<Fq6> = RAM::new(20);
        let mut ram_fq2: RAM<Fq2> = RAM::new(20);
        let mut value: Option<Fq12> = None;
        for round in 0..ELL_ROUNDS_COUNT {
            value = ell_partial(round, &mut ram_fq12, &mut ram_fq2, &mut ram_fq6, coeffs(), g1_affine(), f()).unwrap();
        }
        assert_eq!(value.unwrap(), original_ell(f(), coeffs(), g1_affine()));
    }

    #[test]
    fn test_inverse_fq12() {
        let mut ram_fq6: RAM<Fq6> = RAM::new(20);
        let mut value: Option<Fq12> = None;
        for round in 0..INVERSE_FQ12_ROUNDS_COUNT {
            value = inverse_fq12_partial(round, &mut ram_fq6, f()).unwrap();
        }
        assert_eq!(value.unwrap(), f().inverse().unwrap());
    }

    #[test]
    fn test_exp_by_neg_x() {
        let mut ram_fq12: RAM<Fq12> = RAM::new(20);
        let mut value: Option<Fq12> = None;
        for round in 0..EXP_BY_NEG_X_ROUNDS_COUNT {
            value = exp_by_neg_x_partial(round, &mut ram_fq12, f()).unwrap();
        }
        assert_eq!(value.unwrap(), original_exp_by_neg_x(f()));
    }

    #[test]
    fn test_final_exponentiation() {
        let mut ram_fq12: RAM<Fq12> = RAM::new(40);
        let mut ram_fq6: RAM<Fq6> = RAM::new(20);
        let mut value = None;
        for round in 0..FINAL_EXPONENTIATION_ROUNDS_COUNT {
            value = final_exponentiation_partial(round, &mut ram_fq12, &mut ram_fq6, f()).unwrap();
        }
        assert_eq!(value.unwrap(), Bn254::final_exponentiation(&f()).unwrap());
    }

    // https://github.com/arkworks-rs/algebra/blob/6ea310ef09f8b7510ce947490919ea6229bbecd6/ec/src/models/bn/mod.rs#L59
    fn original_ell(f: Fq12, coeffs: (Fq2, Fq2, Fq2), p: G1Affine) -> Fq12 {
        let mut c0: Fq2 = coeffs.0;
        let mut c1: Fq2 = coeffs.1;
        let c2: Fq2 = coeffs.2;
    
        c0.mul_assign_by_fp(&p.y);
        c1.mul_assign_by_fp(&p.x);

        let mut f = f;
        f.mul_by_034(&c0, &c1, &c2);
        f
    }

    // https://github.com/arkworks-rs/algebra/blob/6ea310ef09f8b7510ce947490919ea6229bbecd6/ec/src/models/bn/mod.rs#L78
    fn original_exp_by_neg_x(f: Fq12) -> Fq12 {
        let mut f = f.cyclotomic_exp(&Parameters::X);
        if !Parameters::X_IS_NEGATIVE { f.conjugate(); }
        f
    }
}