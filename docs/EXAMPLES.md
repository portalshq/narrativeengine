# EXAMPLES.md

`px` is a domain-agnostic entity management system. By using custom entity types and a repository-defined structure, you can adapt `px` to store, track, and manage information in any domain.

The following examples assume a root `repository.yaml` file defining the valid entity types for the project.

---

## 1. Scientific Research Repository
**Domain:** Academic research, tracking experimental data and publication metadata.
**Types:** `paper`, `experiment`, `dataset`

### Setup
```yaml
# repository.yaml
types:
  - paper
  - experiment
  - dataset
```

### Commands
```bash
# Create a new research paper entity
px create paper/cold-fusion-v2 -n "Replicating Cold Fusion Results"

# Assign domain-specific properties
px set px://lab/paper/cold-fusion-v2 status "peer-reviewed"
px set px://lab/paper/cold-fusion-v2 doi "10.1038/example"

# Add representation (the actual PDF)
px add px://lab/paper/cold-fusion-v2 manuscript ./data/manuscript.pdf --format pdf
```

---

## 2. IT Inventory Management
**Domain:** Tracking hardware, software licenses, and their physical locations.
**Types:** `device`, `location`, `vendor`

### Setup
```yaml
# repository.yaml
types:
  - device
  - location
  - vendor
```

### Commands
```bash
# Create a new device entity
px create device/macbook-m4-042 -n "Engineer Laptop 042"

# Assign domain-specific properties
px set px://it/device/macbook-m4-042 serial_number "A123-BC456"
px set px://it/device/macbook-m4-042 assigned_to "jane.doe@company.com"

# Add representation (purchase receipt)
px add px://it/device/macbook-m4-042 purchase_receipt ./docs/receipts/mbp-042.pdf --format pdf
```

---

## 3. World-Building & Lore (Entertainment)
**Domain:** A creative project developing a fantasy world, tracking deities, geography, and historical events.
**Types:** `deity`, `region`, `event`

### Setup
```yaml
# repository.yaml
types:
  - deity
  - region
  - event
```

### Commands
```bash
# Create a new deity entity
px create deity/solaris -n "Solaris, The Sun Bringer"

# Assign domain-specific properties
px set px://fantasy/deity/solaris alignment "Lawful Good"
px set px://fantasy/deity/solaris domain "Light"

# Add representation (character concept art)
px add px://fantasy/deity/solaris concept_art ./assets/art/solaris_concept.png --format png
```

---

## 4. Software Architectural Documentation
**Domain:** Cataloging internal services, API endpoints, and error codes within a microservices ecosystem.
**Types:** `service`, `endpoint`, `error_code`

### Setup
```yaml
# repository.yaml
types:
  - service
  - endpoint
  - error_code
```

### Commands
```bash
# Create a new service entity
px create service/auth-provider -n "Authentication Service"

# Assign domain-specific properties
px set px://platform/service/auth-provider language "Rust"
px set px://platform/service/auth-provider owner "platform-team"

# Add representation (architecture diagram)
px add px://platform/service/auth-provider architecture_diagram ./docs/diagrams/auth-flow.svg --format svg
```

***

### Summary of Workflow
Regardless of the domain, the `px` interaction remains consistent:
1.  **Define:** Add your types to `repository.yaml`.
2.  **Create:** Use `px create <type>/<id>` to initialize an entity.
3.  **Set:** Use `px set <uri> <key> <value>` to attach metadata.
4.  **Represent:** Use `px add <uri> <key> <file> --format <format>` to attach files, assets, or documentation.
