use std::sync::mpsc::Sender;

use block2::RcBlock;
use objc2_foundation::{NSDistributedNotificationCenter, NSRunLoop, NSString};

const PLAYER_INFO_NOTIFICATION: &str = "com.apple.Music.playerInfo";

pub fn notify_on_song_update(tx: Sender<()>) {
    let notifCenter = NSDistributedNotificationCenter::defaultCenter();
    let name = NSString::from_str(PLAYER_INFO_NOTIFICATION);
    let block = RcBlock::new(move |_| {
        tx.send(());
    });
    unsafe {
        notifCenter.addObserverForName_object_queue_usingBlock(Some(&name), None, None, &block);
    }

    NSRunLoop::currentRunLoop().run();
}
