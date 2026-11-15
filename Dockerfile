FROM gcc:14

RUN useradd --uid 10001 --create-home runner

WORKDIR /work

USER runner

CMD ["bash"]
