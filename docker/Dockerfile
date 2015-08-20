FROM debian:8

RUN apt-get update \
 && apt-get install -y openssl

COPY sota_client /bin/sota_client
COPY run.sh /bin/run.sh

EXPOSE 9000
CMD ["/bin/run.sh"]
