FROM rust:1.86 as chef
WORKDIR /usr/src/pulumi-operator
USER root
ENV CARGO_HOME=/root/.cargo
RUN apt update && apt install -y curl python3-pip git

ENV NVM_DIR /usr/local/nvm
RUN mkdir $NVM_DIR
SHELL ["/bin/bash", "--login", "-i", "-c"]
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.5/install.sh | bash
SHELL ["/bin/sh", "-c"]
RUN . $NVM_DIR/nvm.sh && nvm install v16.20.2 && nvm alias default v16.20.2
ENV PATH $NVM_DIR/versions/node/v16.20.2/bin:$PATH

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
RUN cargo install --path pulumi-operator-kubernetes-job && ls -alh /root/.cargo/bin/pulumi-operator-kubernetes-job && cp /root/.cargo/bin/pulumi-operator-kubernetes-job /usr/local/bin/pulumi-operator-kubernetes-job

ENTRYPOINT ["pulumi-operator-kubernetes-job"]
