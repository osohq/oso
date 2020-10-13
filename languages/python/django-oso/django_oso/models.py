"""Django model helpers for authorization."""
from django.db import models
from django.core.exceptions import PermissionDenied

from django_oso.auth import authorize_model


class AuthorizedQuerySet(models.QuerySet):
    """``QuerySet`` with ``authorize()`` method."""

    def authorize(self, request, *, actor=None, action=None):
        """Return a new ``Queryset`` filtered to contain only authorized models.

        .. warning::

            This feature is currently in preview.

        :param actor: The actor making the request. Defaults to ``request.user``.
        :param action: The action to authorize the actor to perform. Defaults to
                        ``request.method``.
        """
        try:
            filter = authorize_model(
                request=request, model=self.model, actor=actor, action=action
            )
        except PermissionDenied:
            return self.none()

        return self.filter(filter)


class AuthorizedModel(models.Model):
    """Use a manager based on ``AuthorizedQuerySet``, allowing the ``authorize()`` method to be used.

    .. warning::

        This feature is currently in preview.
    """

    objects = AuthorizedQuerySet.as_manager()

    class Meta:
        abstract = True
