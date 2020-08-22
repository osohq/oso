use super::*;

pub type InstanceMethods = HashMap<Name, InstanceMethod>;
#[derive(Clone)]
pub struct InstanceMethod(Arc<dyn Fn(&dyn Any, Vec<Term>, &mut Host) -> Arc<dyn ToPolar>>);

impl InstanceMethod {
    pub fn invoke(&self, receiver: &dyn Any, args: Vec<Term>, host: &mut Host) -> Arc<dyn ToPolar> {
        self.0(receiver, args, host)
    }
}

pub trait IntoInstanceMethod<T> {
    fn into_instance_method(self) -> InstanceMethod;
}

impl<T, R> IntoInstanceMethod<T> for fn(&T) -> R
where
    T: 'static,
    R: 'static + ToPolar,
{
    fn into_instance_method(self) -> InstanceMethod {
        InstanceMethod(Arc::new(
            move |receiver: &dyn Any, args: Vec<Term>, _host: &mut Host| {
                assert!(args.is_empty());
                let receiver = receiver
                    .downcast_ref()
                    .expect("incorrect type for receiver");
                Arc::new((self)(receiver))
            },
        ))
    }
}

impl<T, R: ToPolar> IntoInstanceMethod<T> for &'static dyn for<'r> Fn(&'r T) -> R {
    fn into_instance_method(self) -> InstanceMethod {
        InstanceMethod(Arc::new(
            move |receiver: &dyn Any, args: Vec<Term>, _host: &mut Host| {
                assert!(args.is_empty());
                let receiver = receiver.downcast_ref().unwrap();
                Arc::new((self)(receiver))
            },
        ))
    }
}

impl<T, A, R> IntoInstanceMethod<T> for fn(&T, A) -> R
where
    T: 'static,
    A: 'static + FromPolar,
    R: 'static + ToPolar,
{
    fn into_instance_method(self) -> InstanceMethod {
        InstanceMethod(Arc::new(
            move |receiver: &dyn Any, args: Vec<Term>, host: &mut Host| {
                assert_eq!(args.len(), 1);
                let arg = A::from_polar(&args[0], host).unwrap();
                let receiver = receiver.downcast_ref().unwrap();
                Arc::new((self)(receiver, arg))
            },
        ))
    }
}

impl<T, A, R> IntoInstanceMethod<T> for &'static dyn Fn(&T, A) -> R
where
    A: FromPolar,
    R: ToPolar,
{
    fn into_instance_method(self) -> InstanceMethod {
        InstanceMethod(Arc::new(
            move |receiver: &dyn Any, args: Vec<Term>, host: &mut Host| {
                assert_eq!(args.len(), 1);
                let arg = A::from_polar(&args[0], host).unwrap();
                let receiver = receiver.downcast_ref().unwrap();
                Arc::new((self)(receiver, arg))
            },
        ))
    }
}
