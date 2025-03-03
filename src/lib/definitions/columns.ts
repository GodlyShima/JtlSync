import type { Column } from "$lib/types";

export let column_definitions: Column[] = [
  {
    id: "virtuemart_order_id",
    label: "JTL Order ID",
    visible: true,
    required: true,
  },
  {
    id: "order_number",
    label: "Bestellnummer",
    visible: true,
    required: true,
  },
  {
    id: "created_on",
    label: "Erstellt am",
    visible: true,
    format: (value: string) => new Date(value).toLocaleString("de-DE"),
  },
  {
    id: "first_name",
    label: "Vorname",
    visible: true,
  },
  {
    id: "last_name",
    label: "Nachname",
    visible: true,
  },
  {
    id: "order_total",
    label: "Gesamt",
    visible: true,
    format: (value: number) =>
      value.toLocaleString("de-DE", { style: "currency", currency: "EUR" }),
  },
  {
    id: "order_status",
    label: "Status",
    visible: true,
    format: (value: string) => {
      const paymentStatus = value;

      if (paymentStatus === "P") {
        return "Nicht Bezahlt";
      } else if (paymentStatus === "X") {
        return "Stoniert";
      } else {
        return "Bezahlt";
      }
    },
  },
  {
    id: "email",
    label: "E-Mail",
    visible: false,
  },
];
