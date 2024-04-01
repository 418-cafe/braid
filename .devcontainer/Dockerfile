FROM rust:1-bookworm

USER root

# Remove imagemagick due to https://security-tracker.debian.org/tracker/CVE-2019-10131

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive && \
    apt-get purge -y imagemagick imagemagick-6-common && \
    apt-get install -y wget && \
    wget -O - https://debian.neo4j.com/neotechnology.gpg.key | apt-key add - && \
    echo 'deb https://debian.neo4j.com stable 5' | tee -a /etc/apt/sources.list.d/neo4j.list && \
    apt-get update && \
    apt-get install -y neo4j=1:5.18.1

ARG pwd=neo4jjjj

ENV NEO4J_USR=neo4j
ENV NEO4J_PWD=$pwd

RUN /usr/bin/neo4j-admin dbms set-initial-password $pwd
