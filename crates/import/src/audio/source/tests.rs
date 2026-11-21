use super::*;
use crate::audio::tests::blocks;

#[test]
fn resource_lookup_observes_area_overrides_and_refuses_missing_or_noise_samples() {
    let mut sample = [0_u8; 9];
    sample[0] = 3;
    let common_image = blocks(&[
        (0x6c00, &[0, 0x80, 0, 0, 1, 0]),
        (0x6d00, &[0, 0x6e, 0, 0x6e]),
        (0x6e00, &sample),
    ]);
    let common = Upload::parse(&common_image).unwrap();
    let empty = Upload::parse(&[0; 4]).unwrap();
    let mut resources = Resources::new(&common, &empty);
    assert_eq!(resources.instrument(0), Ok(TimbreId(0)));
    assert_eq!(resources.instrument(0), Ok(TimbreId(0)));
    assert_eq!(resources.instrument(1), Err(ImportError::Resource));
    assert_eq!(resources.instruments.len(), 1);
    assert_eq!(
        resources.samples,
        [SampleSource {
            frames: 16,
            loop_from: Some(0),
            source_sha256: sha256(&sample)
        }]
    );

    let noise_image = blocks(&[(0x6c00, &[0x80, 0, 0, 0, 1, 0])]);
    let noise = Upload::parse(&noise_image).unwrap();
    let mut resources = Resources::new(&common, &noise);
    assert_eq!(resources.instrument(0), Err(ImportError::Resource));

    let unavailable_image = blocks(&[(0x6d00, &[9, 0x6e, 9, 0x6e])]);
    let unavailable = Upload::parse(&unavailable_image).unwrap();
    let mut resources = Resources::new(&common, &unavailable);
    assert_eq!(resources.instrument(0), Err(ImportError::Resource));

    let split_loop_image = blocks(&[(0x6d00, &[0, 0x6e, 1, 0x6e])]);
    let split_loop = Upload::parse(&split_loop_image).unwrap();
    let mut resources = Resources::new(&common, &split_loop);
    assert_eq!(resources.instrument(0), Err(ImportError::Resource));
}
