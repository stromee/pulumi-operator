FROM rust:1.86 as chef
WORKDIR /usr/src/pulumi-operator
USER root
ENV CARGO_HOME=/root/.cargo
RUN apt update && apt install -y curl python3-pip npm
RUN curl -fsSL https://get.pulumi.com | sh
ENV PATH="/root/.pulumi/bin:${PATH}"
RUN cargo install cargo-chef

FROM chef as planner
WORKDIR /usr/src/pulumi-operator
USER root
ENV CARGO_HOME=/root/.cargo
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
WORKDIR /usr/src/pulumi-operator
USER root
ENV CARGO_HOME=/root/.cargo
COPY --from=planner /usr/src/pulumi-operator/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo install --path pulumi-operator-kubernetes && ls -alh /root/.cargo/bin/pulumi-operator-kubernetes && cp /root/.cargo/bin/pulumi-operator-kubernetes /usr/local/bin/pulumi-operator-kubernetes

ENTRYPOINT ["pulumi-operator-kubernetes"]
