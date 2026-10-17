//! Shared process-budget controls used by the confined supervisor adapters.
use super::*;
use std::sync::mpsc;

fn deadline(after: Duration) -> Result<u64> {
    crate::bounded::now_unix_ms()?
        .checked_add(u64::try_from(after.as_millis())?)
        .context("test worker deadline overflow")
}

#[test]
fn concurrent_producers_release_partial_reservations_and_hold_their_full_capacity() -> Result<()> {
    let directory = crate::testing::Directory::new()?;
    let root = directory.0.clone();
    let (first, _) = acquire_leases_until(&root, 4, 2, deadline(Duration::from_secs(2))?)?;
    let (started, starting) = mpsc::channel();
    let (acquired, ready) = mpsc::channel();
    let (release, releasing) = mpsc::channel();
    let waiting_root = root.clone();
    let waiting = std::thread::spawn(move || -> Result<()> {
        started.send(())?;
        let (held, _) = acquire_leases_until(&waiting_root, 4, 4, deadline(Duration::from_secs(5))?)?;
        acquired.send(held.len())?;
        releasing.recv()?;
        drop(held);
        Ok(())
    });
    starting.recv_timeout(Duration::from_secs(2))?;
    std::thread::sleep(Duration::from_millis(50));
    // A four-worker producer waiting behind the first two workers must leave
    // the remaining two available to a producer that can actually run.
    let (second, _) = acquire_leases_until(&root, 4, 2, deadline(Duration::from_secs(1))?)?;
    assert!(ready.try_recv().is_err(), "a producer acquired more than the shared four-worker budget");
    drop(second);
    drop(first);
    assert_eq!(ready.recv_timeout(Duration::from_secs(3))?, 4);
    assert!(
        acquire_leases_until(&root, 4, 1, deadline(Duration::from_millis(50))?).is_err(),
        "a fifth worker started while a four-worker producer held the pool"
    );
    release.send(())?;
    waiting.join().expect("worker reservation thread panicked")?;
    let (recovered, _) = acquire_leases_until(&root, 4, 4, deadline(Duration::from_secs(1))?)?;
    assert_eq!(recovered.len(), 4);
    Ok(())
}
