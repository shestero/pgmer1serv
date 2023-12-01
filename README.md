# pgmer1serv
nng &lt;-- HTTP API server for MeritRank (to be used for tests).

This server makes following HTTP endpoints:

* [GET] /  _(general information)_
* [GET] /**edge**/*a1*/*a2*/*3*
* [PUT] /**edge** _(for pgmer1)_
* [GET] /**node_score**/*a1*/*a2*
* [GET] /**scores**/*a1*

It forwards the request to NNG server (such as pgmer2serv).

It is also compartible with pgmer1 PostgreSQL extension.

No Swagger docs yet.
