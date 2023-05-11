use std::mem::size_of;


#[ignore]
#[test]
fn test_morton_ordering() {

    #[derive(Clone, Debug, Default)]
    struct TestVoxel {
        position: (u32, u32, u32),
        enabled: bool
    }

    let n = 50;
    let r = 16;
    let s = n*n*n;

    let mut v = vec![TestVoxel::default(); s];
    let mut morton_v = vec![0u64; s];
    for x in 0..n {
        for y in 0..n {
            for z in 0..n {
                let b = x*x + y*y + z*z < r*r;
                let i = (x * (n*n)) + (y * n) + z;
                v[i] = TestVoxel {
                    position: (x as u32, y as u32, z as u32),
                    enabled: b,
                };
                morton_v[i] = morton_encode_magicbits(x as u32, y as u32, z as u32);
            }
        }
    }

    let mut a: Vec<(&TestVoxel, &u64)> = v.iter().zip(morton_v.iter()).collect();
    a.sort_by_key(|&(&_,&b)| b);

    let mut octree = Octree::new();
    for (voxel, _code) in a {
        let (x, y, z) = voxel.position;
        if voxel.enabled {
            octree.insert(x as usize, y as usize, z as usize);
            assert_eq!(octree.get(x as usize, y as usize, z as usize), true);
        }
    }

    println!("Depth of octree: {}", octree.depth);

    println!("Size of v:{}", size_of::<Vec<TestVoxel>>() + (size_of::<TestVoxel>() * s))
}

#[ignore]
#[test]
fn test_morton_encoding() {
    let x = 0b1100;
    let y = 0b0101;
    let z = 0b1000;

    let expected_m = 0b101011000010;
    assert_eq!(morton_encode(x, y, z), expected_m);
    assert_eq!(morton_encode_magicbits(x, y, z), expected_m);

    assert_eq!(morton_decode(expected_m), (x, y, z));
}
