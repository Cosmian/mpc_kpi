use scale::*;
use crate::bit_protocols::*;

/* This implements arithmetic in Z_<K>
 *  It uses the global statistical security parameter kappa
 * Follows the algorithms in Section 14.3 of the main Scale
 * manual
 *
 * x is held mod p, but it is considered to 
 *   be an integer in the range -2^{k-1},...,2^{k-1}-1
 *
 * Note k is always less than log 2p
 */

#[derive(Copy, Clone)]
pub struct ClearInteger<const K: u64> {
    x: ClearModp,
}

#[derive(Copy, Clone)]
pub struct SecretInteger<const K: u64> {
    x: SecretModp,
}

impl<const K: u64> From<ClearInteger<K>> for SecretInteger<K> {
    #[inline(always)]
    fn from(a: ClearInteger<K>) -> SecretInteger<K> {
        SecretInteger {
          x : SecretModp::from(a.x)
        }
    }
}

impl<const K: u64> From<i64> for ClearInteger<K> {
    #[inline(always)]
    fn from(a: i64) -> ClearInteger<K> {
        ClearInteger {
          x: ClearModp::from(a)
        }
    }
}

impl<const K: u64> ClearInteger<K> {
    #[inline(always)]
    pub fn rep(self) -> ClearModp {
        self.x
    }

}

impl<const K: u64> Reveal for SecretInteger<K> {
  type Output = ClearInteger<K>;
  #[inline(always)]
  fn reveal(&self) -> ClearInteger<K> {
      ClearInteger {
        x: self.x.reveal()
      }
  }
}


impl<const K: u64> ClearInteger<K> {
  #[inline(always)]
  #[allow(non_snake_case)]
  pub fn Mod2m(self, m: u64) -> ClearInteger<K> {
      let twom=modp_two_power(m);
      ClearInteger {
         x: self.x%twom
      }
  }
  #[allow(non_snake_case)]
  pub fn Mod2(self) -> ClearInteger<K> {
      let two = ClearModp::from(2);
      ClearInteger {
         x: self.x%two
      }
  }
}


impl<const K: u64> SecretInteger<K> {
  #[inline(always)]
  #[allow(non_snake_case)]
  pub fn Mod2m(self, m: u64, signed: bool) -> SecretInteger<K> {
      let (r_dprime, r, rb) = PRandM(K, m, unsafe{ KAPPA } );
      let c2m = ClearModp::from(i64_two_power(m));
      let t0 = r_dprime*c2m;
      let mut t1=self.x;
      if signed {
         let twok=ClearModp::from(i64_two_power(K-1));
         t1=t1+twok;
      }
      let t2=t0+t1;
      let t3=t2+r;
      let c=t3.reveal();
      let c_prime=c%c2m;
      let ans: SecretModp;
      if m!=1 {
        let u=BitLT(c_prime, &rb);
        let t4=u*c2m;
        let t5=c_prime-r;
        ans=t5+t4;
      }
      else {
        let tt=c_prime+c_prime;
        ans=c_prime+rb.get(0)-tt*rb.get(0);
      }
      SecretInteger{
        x: ans
      }
  }

  #[inline(always)]
  #[allow(non_snake_case)]
  pub fn Mod2(self, signed: bool) -> SecretInteger<K> {
      self.Mod2m(1, signed)
  }
}





