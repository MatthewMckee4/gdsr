FROM python:3.12-slim

WORKDIR /usr/src/app

RUN pip install --upgrade pip

COPY requirements-dev.txt ./

RUN pip install --no-cache-dir -r requirements-dev.txt

COPY . .

CMD ["bash"]