# Django Lightning Service

make django service Layer fast and simple as light. 

disclamer: this is a work in progress project

this project is meant to help building Service Layer in django as easy and naive as possible, with no pitfall and cost.

it's the response  from experienced dev to the common pitfall and performance problem that come with big applications and businesses logic.


## what do this lib do


- allow you to write your business logic into small and isolated pieces of code
-  wrap the database with a smart layer, making all call as scares as possible.
- propose a neat way to write service with transactional behavior: aggregate you changes, and make the machinery run once
- natively handle batch of changes and optimize database/computation to be run only when it make sens.


## what do this lib don't do

- interpret human readable business rules
- handle asynchronous computation


## this lib is for you if 

- you meet the need to have a service layer: 
   - your business logic is becomming too huge to be exploded between serializers/form/view and models.
   - you duplicate much of your logic between many view or serializers
   - it become harder to know which part is responsible for the business logic
   
- your actual business code is doing lot of database query, often duplicated or similar
- you have performance issus, where a simple view is taking too much time to treat a bunch of changes
- you need to handle batch computation/changes
- your business rules are harder and harder to change because they are intricated
- you sometime hit some cascade changes that end up doing the same computation again and again
- your main problem is during writing, not read of the data
- your model have data tigtly coupled between tables ex: `parent.total = sum(children.amount)`


## this lib is *NOT* for you if 

- you just do CRUD operation, without much logic
- your complexe part is mostly hosted in queryset to build read.


## why Django Ligthning Service 

### the other approach for Service Layer (and their pitfall) 

- View -> ModelSerializer -> Model method / + Signal / + Celery
- View -> (Model)Serializer -> Service Method chained / + Signal / + Celery


- add logic in the view
  - it's the good place for your logic, as long as you don't have more than one view that update the same model
-  add logic in the serializers because it's convenient
  - same as view, it's ok but your serializer will be specialized to be called from a specific view. 
   - if you need to update more than one model in the (Model)serializer, it's  not going to be the good place anymore
 
-  react to change via the post_save signal
  - it's ok as long as you call your model.save() only once.
  - the post_save does not have the inital value, nor which value is updated, so if you need to react to a change for a specific attrubute, your in the pit
  - overall, you lack the context of your change. maybe your model is updated by the shell ? maybe it's updated with another model that will not be visible during the signal handling ?
  - the signal is not triggered by batch_update/batch_create

- where do you call save() ?  who is responsible for this call ?
  - if your serializer need to change more than one model, he will need to call save multiple time. 
  - but what if you need to re-update the same model later via another serializer/view ? we will call save twice
 
 - use select_related/prefetch_related to help with duplicate/similar db query
  - this work as long as you think about updating the queryset when you update the serializer
  - this work as long as you don't add any `filter()` in the related manager
  - this work as long as you'r code is not shared between views that way not add the select/prefetch 
 
 - nested serializer may not see the others done/pending changes
   - the most impactfull pitfall we had is where your view will update multiple model, 
     and each model should trigger a cascade. during the cascade of effect, 
	 you will not see the changes not yet saved() in the database, and you will probably hit the database more than once

overall, all of these pitfall can be mitigated via some more code and logic, which lead to complexe and techincal code along with the business logic, often making it harder and harder to read and change.

	 
 ### Django Ligthning Service strucural choices
 
 - we wrap all call on the database and add heuristic to prevent duplicate/similar call to it
 - all entity returned by the wrapper is singleton for a given Orm instance. you fetch twice the same order ? you get the SAME order model, which btw may be already changed by the previous code
-  we provide an engine to support the [Command Pattern](https://refactoring.guru/design-patterns/command) . this engine will allow to aggregate all requested changed without running them, and once everything is asked, run them.
  - this approach will heavily help to optimize database call because before we do any logic, we already know which entity will be required to run, and we will be able to batch most of the fetch.
- this same engine will allow to write module that react to changes, aka a [Observer Pattern](https://refactoring.guru/design-patterns/observer) 

with this small set of component, we can create really fast and efficient services that allow to 

- batch lot of data blazingly fast
- write business data without noise about performance complication
- ensure to have the context of a change when reacting to it (you have the data before and after the change)
- batch_create/batch_update/batch_delete without thinking about it
- test your business code unit per unit
- track validation error up to the root value that caused the error. 


## ok, but what does it look like in real ?
 
 - all logic is hosted into a service, himself hosting common behavior/rules into dedicated pieces of code called Module that react to changes on the models
 - the service is initialized by the view, and is provided to underneath serializers
 - the serializers call the public method of the save to «ask for changes», without any other logic
 - once the view is done with the serializer, it call `service.commit()` to ask for the service to run all the queued changes. 
 - the service will apply all changes, and execute all modules that should run given the changes asked
 - the view interpret the result of `commit()` to know if it was a success, and if yes, it return the expected response
 - if the commit() had an error, the result will contains the details of which part of the changes was erroreneous and the serilizer transform it into a ValidationError
 
 TODO: add a sample project
 
 



