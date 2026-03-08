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
                pub struct [<HTTP $service_name $gate_name Req>] {
                    $( pub $req_field : $req_ty ),*
                }
                impl $crate::Payload for [<HTTP $service_name $gate_name Req>] { }

                // Gate response struct
                #[derive($crate::serde::Serialize, $crate::serde::Deserialize, Clone, Default)]
                pub struct [<HTTP $service_name $gate_name Res>] {
                    $( pub $res_field : $res_ty ),*
                }
                impl $crate::Payload for [<HTTP $service_name $gate_name Res>] { }

                impl From<[<HTTP $service_name $gate_name Res>]> for [<HTTP $service_name ResGates>] {
                    fn from(item: [<HTTP $service_name $gate_name Res>]) -> Self {
                        [<HTTP $service_name ResGates>]::$gate_name(item)
                    }
                }

                impl From<[<HTTP $service_name ReqGates>]> for [<HTTP $service_name $gate_name Req>] {
                    fn from(value: [<HTTP $service_name ReqGates>]) -> Self {
                        let gate = if let [<HTTP $service_name ReqGates>]::$gate_name(req) = value {
                            Ok(req)
                        } else {
                            Err(())
                        };

                        gate.map_err(|()| $crate::error!("Gate conversion failed"))
                            .unwrap()
                    }
                }
                
                impl From<[<HTTP $service_name $gate_name Req>]> for [<HTTP $service_name ReqGates>] {
                    fn from(value: [<HTTP $service_name $gate_name Req>]) -> Self {
                        [<HTTP $service_name ReqGates>]::$gate_name(value)
                    }
                }
            )*

            // Request gates enum
            #[derive(Clone, $crate::serde::Serialize, $crate::serde::Deserialize)]
            #[serde(tag = "gate")]
            pub enum [<HTTP $service_name ReqGates>] {
                $(
                    $gate_name([<HTTP $service_name $gate_name Req>])
                ),*
            }
            impl $crate::Payload for [<HTTP $service_name ReqGates>] { }
            impl $crate::http::RequestGatesMarker for [<HTTP $service_name ReqGates>] { }


            // Response gates enum
            #[derive(Clone, $crate::serde::Serialize, $crate::serde::Deserialize)]
            #[serde(tag = "gate")]
            pub enum [<HTTP $service_name ResGates>] {
                $(
                    $gate_name([<HTTP $service_name $gate_name Res>])
                ),*
            }
            impl $crate::Payload for [<HTTP $service_name ResGates>] { }
            impl $crate::http::ResponseGatesMarker for [<HTTP $service_name ResGates>] { }

            pub struct $service_name { }
            impl $crate::http::Service for $service_name {
                type Requests = [<HTTP $service_name ReqGates>];
                type Responses = [<HTTP $service_name ResGates>];
            }

            #[macro_export]
            macro_rules! [<$service_name:snake _binder>] {
                [ $worker:ident | $request:ident ] => {
                    match $request {
                        $(
                            [<HTTP $service_name ReqGates>]::$gate_name(req) => 
                                [<HTTP $service_name $gate_name Serv>]::process(req, $worker)
                                    .await
                                    .map(|res| Into::<[<HTTP $service_name ResGates>]>::into(res))
                        ),*
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! http_bind_service {
    {
        $service_name:ident
    } => {
        $crate::paste::paste! {
            type S = $service_name;

            async fn matcher(&self, request: <Self::S as $crate::http::Service>::Requests)
                -> Result<<Self::S as $crate::http::Service>::Responses, $crate::Error> 
            { [<$service_name:snake _binder>]![ self | request ] }
        }
    };
}