# Stacks
The stack domain is the part of Brume managing all clients infrastructures. 
Brume needs to have priviledge access to client accounts to be able to create the stack on users's account.

## Timeline 
## AWS 
1. The log in the account they want to allow Brume access on.
2. Brume redirects to https://eu-west-1.console.aws.amazon.com/cloudformation/home?region=eu-west-1#/stacks/quickcreate?templateURL=STACK_JSON&stackName=BrumeRole&param_TrustArnParameter=arn:aws:iam::401399516766:role/BrumeUserAssume
3. User "check the connection"
4. Everything is set