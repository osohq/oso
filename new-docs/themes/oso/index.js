import("oso").catch(e =>
    console.error("Error importing `oso`:", e)
).then(m => window.oso = m)
