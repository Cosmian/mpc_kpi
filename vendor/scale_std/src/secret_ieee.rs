use core::mem;
use core::ops::{Mul,Add,Div,Neg,Sub};
use scale::*;
use crate::circuits::*;

// The secure IEEE Type
#[derive(Copy, Clone)]
pub struct SecretIEEE {
   val: SecretI64,
}


// The holding "insecure" IEEE Type
//   - This variant allows no arithmetic just 
//     allows parsing and printing
#[derive(Copy, Clone)]
pub struct ClearIEEE {
    val: i64,
}

impl From<f64> for ClearIEEE {
    #[inline(always)]
    fn from(f: f64) -> ClearIEEE {
        unsafe { mem::transmute(f) }
    }
}


// Get the bit representation in an i64 
impl ClearIEEE {
    #[inline(always)]
    pub fn rep(self) -> i64 {
        self.val
    }
}

impl ClearIEEE {
    #[inline(always)]
    pub fn print(self) {
        unsafe { __print_ieee_float(self.val) }
    }
}

/* XXXX OLIVER PLEASE CHECK THERE IS NOT A BETTER WAY */
impl From<ClearIEEE> for SecretIEEE {
    #[inline(always)]
    fn from(c: ClearIEEE) -> SecretIEEE {
       SecretIEEE {
          val : unsafe {
            __convregsreg(c.val)
          } 
       }
    }
}

/* XXXX OLIVER PLEASE CHECK THERE IS NOT A BETTER WAY */
impl From<SecretI64> for SecretIEEE {
    #[inline(always)]
    fn from(s: SecretI64) -> SecretIEEE {
        SecretIEEE {
          val : IEEE_i2f(s)
        }
    }
}


/* XXXX OLIVER PLEASE CHECK THERE IS NOT A BETTER WAY */
impl From<i64> for SecretIEEE {
    #[inline(always)]
    fn from(i: i64) -> SecretIEEE {
        let s = SecretI64::from(i);
        SecretIEEE {
          val : IEEE_i2f(s)
        }
    }
}


/* XXXX OLIVER PLEASE CHECK THERE IS NOT A BETTER WAY */
impl From<SecretIEEE> for SecretI64 {
    #[inline(always)]
    fn from(f: SecretIEEE) -> SecretI64 {
        IEEE_f2i(f.val)
    }
}


/* XXXX OLIVER PLEASE CHECK THERE IS NOT A BETTER WAY */
impl Reveal for SecretIEEE {
    type Output = ClearIEEE;
    #[inline(always)]
    fn reveal(&self) -> ClearIEEE {
      ClearIEEE {
        val: self.val.reveal()
      }
    }
}


/*
impl SecretCmp<Self> for SecretIEEE {
    #[inline(always)]
    fn lt(self, other: Self) -> SecretBit {
        let ans = IEEE_lt([self.val, other.val]);
        __convsintsbit(ans)
    }
    #[inline(always)]
    fn le(self, other: Self) -> SecretBit {
        self.lt(other) | self.eq(other)
    }
    #[inline(always)]
    fn gt(self, other: Self) -> SecretBit {
        let ans = IEEE_lt([other.val, self.val]);
        __convsintsbit(ans)
    }
    #[inline(always)]
    fn ge(self, other: Self) -> SecretBit {
        self.gt(other) | self.eq(other)
    }
    #[inline(always)]
    fn eq(self, other: Self) -> SecretBit {
        let ans = IEEE_eq([other.val, self.val]);
        __convsintsbit(ans)
    }
    #[inline(always)]
    fn ne(self, other: Self) -> SecretBit {
        !self.eq(other)
    }
}
*/


/* XXXX OLIVER PLEASE CHECK THERE IS NOT A BETTER WAY */
impl Mul<SecretIEEE> for SecretIEEE {
    type Output = Self;
    #[inline(always)]
    fn mul(self, other: SecretIEEE) -> Self::Output {
       SecretIEEE {
          val : IEEE_mul([self.val, other.val]) 
       }
    }
}


impl Add<SecretIEEE> for SecretIEEE {
    type Output = Self;
    #[inline(always)]
    fn add(self, other: SecretIEEE) -> Self::Output {
       SecretIEEE {
          val : IEEE_add([self.val, other.val])
       }
    }
}


impl Div<SecretIEEE> for SecretIEEE {
    type Output = Self;
    #[inline(always)]
    fn div(self, other: SecretIEEE) -> Self::Output {
       SecretIEEE {
          val : IEEE_div([self.val, other.val])
       }
    }
}


impl Neg for SecretIEEE {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self::Output {
       SecretIEEE {
          val : unsafe{  
            __xorsintc(self.val, 1_i64<<63) 
          }
       }
    }
}


impl Sub<SecretIEEE> for SecretIEEE {
    type Output = Self;
    #[inline(always)]
    fn sub(self, other: SecretIEEE) -> Self::Output {
       SecretIEEE {
          val : IEEE_add([self.val, (-other).val])
       }
    }
}


impl SecretIEEE {
    #[inline(always)]
    pub fn sqrt(self) -> SecretIEEE {
       SecretIEEE {
          val : IEEE_sqrt(self.val)
       }
    }

}



