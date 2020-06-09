import inspect
from polar import Polar


def add_model(model):
    # fields = tuple(field.name for field in model._meta.get_fields())
    # methods = tuple(name for name, meth in inspect.getmembers(model, predicate=inspect.isfunction)
    #         # TODO(gj): what about when the application developer shadows a built-in Model method, like a custom `Model.save()` implementation?
    #         if not meth.__module__ == 'django.db.models.base'
    #         and not name.startswith('_'))

    def from_polar(kwargs_dict=None, *, model=model, **actual_kwargs):
        if kwargs_dict:
            return model.objects.get(**kwargs_dict)
        else:
            return model.objects.get(**actual_kwargs)

    setattr(model, "_from_polar", from_polar)
    Polar().register_python_class(model, from_polar=model._from_polar)
