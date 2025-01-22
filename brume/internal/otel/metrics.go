package brume_otel

import (
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/metric"
)

var meter = otel.Meter("brume")

var (
	HealthyJobsGauge   metric.Int64Gauge
	UnhealthyJobsGauge metric.Int64Gauge
	TotalJobsGauge     metric.Int64Gauge
)

func Init() {
	HealthyJobsGauge, _ = meter.Int64Gauge("healthy_jobs", metric.WithDescription("Number of healthy jobs"), metric.WithUnit("{jobs}"))
	UnhealthyJobsGauge, _ = meter.Int64Gauge("unhealthy_jobs", metric.WithDescription("Number of unhealthy jobs"), metric.WithUnit("{jobs}"))
	TotalJobsGauge, _ = meter.Int64Gauge("total_jobs", metric.WithDescription("Total number of jobs"), metric.WithUnit("{jobs}"))
}
