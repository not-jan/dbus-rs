extern crate dbus;

#[allow(dead_code)]
mod policykit;

use std::sync::atomic::*;

impl policykit::OrgFreedesktopDBusIntrospectable for () {
   type Err = dbus::tree::MethodErr;
   fn introspect(&self) -> Result<String, Self::Err> { Ok("I feel so introspected right now".into()) }
}

#[test]
fn test_main() {
    let f = dbus::tree::Factory::new_fn::<()>();
    let i1 = policykit::org_freedesktop_dbus_introspectable_server(&f, (), |minfo| minfo.path.get_data());
    let t = f.tree(()).add(f.object_path("/test", ()).add(i1));
    let c = dbus::ffidisp::Connection::new_session().unwrap();
    t.set_registered(&c, true).unwrap();
    let cname = c.unique_name();
    let quit = std::sync::Arc::new(AtomicBool::new(false));
    let quit2 = quit.clone();
    let _ = std::thread::spawn(move || {
        // Old way
        let c2 = dbus::ffidisp::Connection::new_session().unwrap();
        let m = dbus::Message::new_method_call(&cname, "/test", "org.freedesktop.DBus.Introspectable", "Introspect").unwrap();
        let mut mrep = c2.send_with_reply_and_block(m, 1000).unwrap();
        let m2 = mrep.as_result().unwrap();
        assert_eq!(m2.read1(), Ok("I feel so introspected right now"));

        // New way
        let p = c2.with_path(cname, "/test", 1000);
        use policykit::OrgFreedesktopDBusIntrospectable;
        assert_eq!(p.introspect().unwrap(), "I feel so introspected right now");

        quit2.store(true, Ordering::SeqCst);
    }); 
    for _ in t.run(&c, c.iter(100)) { if quit.load(Ordering::SeqCst) { break; } }
/*     */
}
