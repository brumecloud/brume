VERSION=0.122.1
OS=linux_amd64
URL=https://github.com/open-telemetry/opentelemetry-collector-releases/releases/download/v${VERSION}/otelcol-contrib_${VERSION}_${OS}.tar.gz
HERE=$(pwd)

echo "Downloading otel-bin from ${URL}"

curl -L -o /tmp/otelcol-contrib_${VERSION}_${OS}.tar.gz ${URL}

echo "Extracting otel-bin"

mkdir -p /tmp/otelcol

tar -xzvf /tmp/otelcol-contrib_${VERSION}_${OS}.tar.gz -C /tmp/otelcol

echo "Moving otel-bin to ${HERE}/bin/otelcol"

mkdir -p ${HERE}/bin
mv /tmp/otelcol/otelcol-contrib ${HERE}/bin/otelcol

echo "Cleaning up"

rm /tmp/otelcol-contrib_${VERSION}_${OS}.tar.gz
rm -rf /tmp/otelcol
