## Simple blog

To run this project, follow these steps:

* build the container:

```shell
docker build -t jetbrains-blog .
```

* run the container:

```shell
docker run -p 3000:3000 jetbrains-blog
```

You can now access http://localhost:3000/home to try the features of the app.