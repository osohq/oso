import inspect
from polar import Polar


def add_model(model):
    def from_polar(kwargs_dict=None, *, model=model, **actual_kwargs):
        if kwargs_dict:
            return model.objects.get(**kwargs_dict)
        else:
            return model.objects.get(**actual_kwargs)

    setattr(model, "_from_polar", from_polar)
    Polar().register_class(model, from_polar=model._from_polar)
