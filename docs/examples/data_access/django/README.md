# Example

## Setup

- `pip install -r requirements.txt`
- `python example/manage.py makemigrations app`
- `python example/manage.py migrate`
- `python example/manage.py seed`
- `python example/manage.py runserver`

## Usage

- Guests may view public posts:

  ```console
  $ curl localhost:8000/posts
  1 - @user - public - public user post
  3 - @manager - public - public manager post
  ```

- Non-managers may view public posts and their own private posts:

  ```console
  $ curl --user user:user localhost:8000/posts
  1 - @user - public - public user post
  2 - @user - private - private user post
  3 - @manager - public - public manager post
  ```

- Managers may view public posts, their own private posts, and private posts
  of their direct reports:

  ```console
  $ curl --user manager:manager localhost:8000/posts
  1 - @user - public - public user post
  2 - @user - private - private user post
  3 - @manager - public - public manager post
  4 - @manager - private - private manager post
  ```
