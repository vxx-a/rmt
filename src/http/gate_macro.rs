/** *http_gates!*

    Generates gates
    ```
    http_gates! ( MyService [
        Ping {
            request: { ... },
            response: { ... }
        },
        Time {
            request: { ... },
            response: { ... }
        }
    ])
    ```

    Generate Service type with provided name

    Generates additional auxilary binding macro
 */
#[macro_export]
macro_rules! http_gates {
    (
        $service_name:ident [
            $(
                $gate_name:ident {
                    request: { $($req_field:ident : $req_ty:ty),* $(,)? },
                    response: { $($res_field:ident : $res_ty:ty),* $(,)? }
                }
            ),* $(,)?
        ]
    ) => {
        $crate::paste::paste! {
            $(
                // Gate request struct
                #[derive($crate::serde::Serialize, $crate::serde::Deserialize, Clone)]
                pub struct [<RMTHTTP $service_name $gate_name Req>] {
                    $( pub $req_field : $req_ty ),*
                }
                impl $crate::Payload for [<RMTHTTP $service_name $gate_name Req>] { }

                // Gate response struct
                #[derive($crate::serde::Serialize, $crate::serde::Deserialize, Clone)]
                pub struct [<RMTHTTP $service_name $gate_name Res>] {
                    $( pub $res_field : $res_ty ),*
                }
                impl $crate::Payload for [<RMTHTTP $service_name $gate_name Res>] { }

                impl From<[<RMTHTTP $service_name $gate_name Res>]> for [<RMTHTTP $service_name ResGates>] {
                    fn from(item: [<RMTHTTP $service_name $gate_name Res>]) -> Self {
                        [<RMTHTTP $service_name ResGates>]::$gate_name(item)
                    }
                }

                impl From<[<RMTHTTP $service_name ReqGates>]> for [<RMTHTTP $service_name $gate_name Req>] {
                    fn from(value: [<RMTHTTP $service_name ReqGates>]) -> Self {
                        let gate = if let [<RMTHTTP $service_name ReqGates>]::$gate_name(req) = value {
                            Ok(req)
                        } else {
                            Err(())
                        };

                        gate.map_err(|()| $crate::error!("Gate conversion failed"))
                            .unwrap()
                    }
                }
                
                impl From<[<RMTHTTP $service_name $gate_name Req>]> for [<RMTHTTP $service_name ReqGates>] {
                    fn from(value: [<RMTHTTP $service_name $gate_name Req>]) -> Self {
                        [<RMTHTTP $service_name ReqGates>]::$gate_name(value)
                    }
                }
            )*

            // Request gates enum
            #[derive(Clone, $crate::serde::Serialize, $crate::serde::Deserialize)]
            #[serde(tag = "gate")]
            pub enum [<RMTHTTP $service_name ReqGates>] {
                $(
                    $gate_name([<RMTHTTP $service_name $gate_name Req>])
                ),*
            }
            impl $crate::Payload for [<RMTHTTP $service_name ReqGates>] { }
            impl $crate::http::RequestGatesMarker for [<RMTHTTP $service_name ReqGates>] { }


            // Response gates enum
            #[derive(Clone, $crate::serde::Serialize, $crate::serde::Deserialize)]
            #[serde(tag = "gate")]
            pub enum [<RMTHTTP $service_name ResGates>] {
                $(
                    $gate_name([<RMTHTTP $service_name $gate_name Res>])
                ),*
            }
            impl $crate::Payload for [<RMTHTTP $service_name ResGates>] { }
            impl $crate::http::ResponseGatesMarker for [<RMTHTTP $service_name ResGates>] { }

            pub struct $service_name { }
            impl $crate::http::Service for $service_name {
                type Requests = [<RMTHTTP $service_name ReqGates>];
                type Responses = [<RMTHTTP $service_name ResGates>];
            }

            #[macro_export]
            macro_rules! [<$service_name:snake _binder__>] {
                [ $worker:ident | $request:ident ] => {
                    match $request {
                        $(
                            [<RMTHTTP $service_name ReqGates>]::$gate_name(req) =>
                                <[<RMTHTTP $service_name $gate_name Req>] as $crate::http::Gate>::process(req, $worker)
                                    .await
                                    .map(|res| Into::<[<RMTHTTP $service_name ResGates>]>::into(res))
                        ),*
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! http_bind_worker {
    {
        $context:ident | $service_name:ident
    } => {
        $crate::paste::paste! {
            type S = $service_name;

            fn context_ref(&self) -> &'static $crate::http::Context<Self::S> {
                &$context
            }

            async fn matcher(&self, request: <Self::S as $crate::http::Service>::Requests)
                -> Result<<Self::S as $crate::http::Service>::Responses, $crate::Error> 
            { [<$service_name:snake _binder__>]![ self | request ] }
        }
    };
}