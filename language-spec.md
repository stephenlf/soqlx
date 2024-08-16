# SOQLx Language Specification

SOQLx is a _superset_ of traditional SOQL, meaning [all valid SOQL](https://developer.salesforce.com/docs/atlas.en-us.soql_sosl.meta/soql_sosl/sforce_api_calls_soql.htm) is also valid SOQLx. On top of traditional SOQL, SOQLx offers a number of __superpowers__, including:

* Native `IN` clauses
> ```sql
> SELECT Name
> FROM Account
> WHERE ShippingState IN ('OH', 'NY')
> ```
* `DESCRIBE` calls
> ```sql
> DESCRIBE Account
> ```
* Subqueries across lookup fields
> ```sql
> SELECT Name,
>   ( SELECT Name FROM OpportunityLineItems )   // Regular subquery
>   ( SELECT Name FROM Account.Contacts )       // Cross-lookup subquery
> FROM Opportunity
> ```
* Leading `FROM` clause, allowing for powerful autocompletion
> ```sql
> FROM Account
> SELECT
> // ... Let the editor suggest fields, lookup relationships, or child objects
> ```
* Aggregated subqueries and `HAVING` clauses in subqueries
> ```sql
> SELECT Name,
>   ( SELECT count(Id) FROM Contacts HAVING count(Id) > 1)
> FROM Account
> ```
* Native support for bind variables
> ```sql
> // @param myDate 2024-01-03
> // @paramFile 'parameters.yml' myParamFile 
> 
> SELECT Name
> FROM Count
> WHERE ClosedDate > :myDate
>    AND CreatorName IN :myParamFile.creatorNames
> ```

All of these powerful features are available straight out of your editor or the SOQLx CLI. Additionally, you can integrate these queries directly into your Apex code by _transpiling_ the SOQLx into concise and efficient Apex code.
> ```sql
> // soqlx
> FROM Account
> SELECT Name, OwnerId
> ```
> becomes...
> ```java
> // Apex
> Account[] accounts = [
>   SELECT Name,
>     OwnerId
>   FROM Account
> ];
> ```

## Keywords
`AND`
`ASC`
`DESC`
`EXCLUDES`
`FIRST`
`FROM`
`GROUP`
`HAVING`
`IN`
`INCLUDES`
`LAST`
`LIKE`
`LIMIT`
`NOT`
`NULL`
`NULLS`
`OR`
`SELECT`
`USING`
`WHERE`
`WITH`