# ring E2E Benchmark

测试 ring 密码库中 HKDF 的 unsafe API 替换性能提升。

## 替换内容

在 `ring/src/hkdf.rs` 中，将 `fill_okm` 中的 `split_at_mut()` 替换为了 `split_at_mut_unchecked()`，将 `checked_add(1).unwrap()` 替换为了 `unchecked_add(1)`。

`fill_okm` 被包装为 `Okm::fill`。

`nl_wallet` 的 `hkdf` 的实现中直接调用了它。

https://github.com/MinBZK/nl-wallet/blob/a2fb6a6ccd299e89ce382ec75a5a8ee54c5d70af/wallet_core/lib/crypto/src/utils.rs#L30

## 测试代码

直接将`nl_wallet` 的 `hkdf` 实现复制到了本地测试，测试代码如下，使用了 `criterion` 进行测试。

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ring::hkdf;

pub fn hkdf(input_key_material: &[u8], salt: &[u8], info: &str, len: usize) -> Vec<u8> {
    struct HkdfLen(usize);
    impl hkdf::KeyType for HkdfLen {
        fn len(&self) -> usize { self.0 }
    }
    
    let mut bts = vec![0u8; len];
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, salt);
    salt.extract(input_key_material)
        .expand(&[info.as_bytes()], HkdfLen(len))
        .unwrap()
        .fill(bts.as_mut_slice())
        .unwrap();
    bts
}

fn bench_hkdf(c: &mut Criterion) {
    let ikm = b"input key material for testing";
    let salt = b"random salt value";
    let info = "application info";
    
    c.bench_function("hkdf-2048B", |b| {
        b.iter(|| {
            black_box(hkdf(
                black_box(ikm),
                black_box(salt),
                black_box(info),
                2048,
            ))
        })
    });
}

criterion_group!(benches, bench_hkdf);
criterion_main!(benches);
```


```toml
[package]
name = "ring-e2e-benchmark"
version = "0.1.0"
edition = "2021"

[dependencies]
ring = { path = "/tmp/ring-unsafe" }
criterion = { version = "0.5", default-features = false }

[[bench]]
name = "e2e_compare"
harness = false

```


## 结果

提升了 9.3%

unsafe：
```json
{
  "mean": {
    "confidence_interval": {
      "confidence_level": 0.95,
      "lower_bound": 9795.47932012854,
      "upper_bound": 9803.800631146853
    },
    "point_estimate": 9799.390246689849,
    "standard_error": 2.1276662662961305
  },
  "median": {
    "confidence_interval": {
      "confidence_level": 0.95,
      "lower_bound": 9791.691654879774,
      "upper_bound": 9797.334028139656
    },
    "point_estimate": 9795.059828017664,
    "standard_error": 1.4634219352554299
  },
  "median_abs_dev": {
    "confidence_interval": {
      "confidence_level": 0.95,
      "lower_bound": 9.570035761128473,
      "upper_bound": 15.52296279669993
    },
    "point_estimate": 13.339165376282281,
    "standard_error": 1.566596422054961
  },
  "slope": {
    "confidence_interval": {
      "confidence_level": 0.95,
      "lower_bound": 9796.490487060515,
      "upper_bound": 9807.681807339737
    },
    "point_estimate": 9801.671781373498,
    "standard_error": 2.8652738247297163
  },
  "std_dev": {
    "confidence_interval": {
      "confidence_level": 0.95,
      "lower_bound": 14.58128648698178,
      "upper_bound": 27.108477186984707
    },
    "point_estimate": 21.387807853749962,
    "standard_error": 3.204855118384173
  }
}
```


safe:
```json
{
  "mean": {
    "confidence_interval": {
      "confidence_level": 0.95,
      "lower_bound": 10696.721717077817,
      "upper_bound": 10742.11698422521
    },
    "point_estimate": 10718.382401809911,
    "standard_error": 11.603834894165452
  },
  "median": {
    "confidence_interval": {
      "confidence_level": 0.95,
      "lower_bound": 10660.706720430107,
      "upper_bound": 10674.21067303863
    },
    "point_estimate": 10669.465238523253,
    "standard_error": 3.175115045330206
  },
  "median_abs_dev": {
    "confidence_interval": {
      "confidence_level": 0.95,
      "lower_bound": 15.194600052821512,
      "upper_bound": 29.46728991293677
    },
    "point_estimate": 23.45463543988573,
    "standard_error": 3.5422014881787196
  },
  "slope": {
    "confidence_interval": {
      "confidence_level": 0.95,
      "lower_bound": 10738.432249851403,
      "upper_bound": 10812.026685331211
    },
    "point_estimate": 10776.219851779111,
    "standard_error": 18.788177602407085
  },
  "std_dev": {
    "confidence_interval": {
      "confidence_level": 0.95,
      "lower_bound": 91.20741949635998,
      "upper_bound": 136.24761365204967
    },
    "point_estimate": 116.15787824973356,
    "standard_error": 11.52186651310957
  }
}
```
