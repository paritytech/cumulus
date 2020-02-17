FROM python:3-buster

RUN pip install substrate-interface

COPY ./scripts/register_parachain.py /usr/bin/
CMD [ "/usr/bin/register_parachain.py" ]
