input: wallet address, chain, hop, path to custom risk list
Output will just be a top level summary of all risky wallet addresses found
the data included in the output:

- wallet address
- what hop it was found
- risk level
- description

  regarding scoring logic this is quite tough and i cant really come up with a
  standard right now, but i came up with the following which would technically
  be a score out of 3.
  scoring logic:

- Directly Sanctioned wallet
- Direct Mixer / Suspect wallet
- Suspected to be a sanctioned / mixer / suspect (risk) wallet
