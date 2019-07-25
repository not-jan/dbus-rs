extern crate dbus;

use std::sync::atomic::*;
use std::rc::Rc;
use std::convert::AsRef;

#[allow(dead_code)]
#[deny(trivial_casts)]
mod policykit_asref;

struct Whatever {}

impl AsRef<policykit_asref::OrgFreedesktopDBusProperties<Err = ::dbus::tree::MethodErr> + 'static> for Rc<Whatever> {
    fn as_ref(&self) -> &(policykit_asref::OrgFreedesktopDBusProperties<Err = ::dbus::tree::MethodErr> + 'static) { &**self }
}

impl policykit_asref::OrgFreedesktopDBusProperties for Whatever {
    type Err = ::dbus::tree::MethodErr;

    fn get(&self, interfacename: &str, propertyname: &str) -> Result<::dbus::arg::Variant<Box<::dbus::arg::RefArg>>, Self::Err> {
        assert_eq!(interfacename, "Interface.Name");
        assert_eq!(propertyname, "Property.Name");
        Ok(::dbus::arg::Variant(Box::new(5u8)))
    }

    fn get_all(&self, _interfacename: &str) -> Result<::std::collections::HashMap<String, ::dbus::arg::Variant<Box<::dbus::arg::RefArg>>>, Self::Err> { unimplemented!() }

    fn set(&self, _interfacename: &str, _propertyname: &str, value: ::dbus::arg::Variant<Box<::dbus::arg::RefArg>>) -> Result<(), Self::Err> {
        assert_eq!((&value as &dbus::arg::RefArg).as_str(), Some("Hello")); 
        Err(("A.B.C", "Error.Message").into())
    }

}


#[test]
fn test_asref() {
    let f = dbus::tree::Factory::new_fnmut::<()>();
    let x = Rc::new(Whatever {});
    let i1 = policykit_asref::org_freedesktop_dbus_properties_server(&f, (), move |_| { x.clone() });
    let t = f.tree(()).add(f.object_path("/test", ()).add(i1));
    let c = dbus::ffidisp::Connection::new_session().unwrap();
    t.set_registered(&c, true).unwrap();
    let cname = c.unique_name();
    let quit = std::sync::Arc::new(AtomicBool::new(false));
    let quit2 = quit.clone();
    let _ = std::thread::spawn(move || {
        use policykit_asref::OrgFreedesktopDBusProperties;
        use dbus::arg::RefArg;

        let c2 = dbus::ffidisp::Connection::new_session().unwrap();
        let p = c2.with_path(cname, "/test", 1000);
        let v = p.get("Interface.Name", "Property.Name").unwrap();
        assert_eq!(v.as_i64(), Some(5));

        let vv = p.set("Interface.Name", "Property.Name", dbus::arg::Variant(Box::new("Hello".to_string())));
        assert_eq!(vv.unwrap_err().message(), Some("Error.Message"));

        quit2.store(true, Ordering::SeqCst);
    }); 
    for _ in t.run(&c, c.iter(100)) { if quit.load(Ordering::SeqCst) { break; } }
}
