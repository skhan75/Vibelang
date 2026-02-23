# Pilot App: Service Reference

This pilot models a small concurrent batch service:

- accepts a list of integer work items,
- fan-outs jobs through channels,
- returns processed results via channel collection.

Entry point: `pilot-apps/service_reference/main.yb`
