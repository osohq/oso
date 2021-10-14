from base64 import urlsafe_b64decode

from django.http import HttpResponse

from app.models import Post, User


def user_from_request(request):
    try:
        username = (
            urlsafe_b64decode(request.headers.get("Authorization").split(" ")[1])
            .decode("utf-8")
            .split(":")[0]
        )
        return User.objects.get(username=username)
    except:
        return User(username="guest")


def index(request):
    request.user = user_from_request(request)
    authorized_posts = Post.objects.authorize(request)
    formatted = [
        f"{post.pk} - @{post.creator.username} - {post.access_level} - {post.contents}"
        for post in authorized_posts
    ]
    return HttpResponse("\n".join(formatted) + "\n", content_type="text/plain")
