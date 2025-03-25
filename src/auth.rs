use std::sync::Arc;

use axum::{extract::{Request, State}, middleware::Next, response::Response};
use jsonwebtoken::{decode, decode_header, Validation};
use jwks::Jwks;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub aud: String,
    pub iss: String,
    pub exp: usize,
    pub iat: usize,
}

pub async fn authentication_middleware(State(state): State<Arc<AppState>>, request: Request, next: Next) -> Result<Response, StatusCode>{
    // Get the Authorization header
    match request.headers().get("Authorization"){
        Some(auth_header) => {
            // Convert Authorization header value to a str reference
            match auth_header.to_str() {
                Ok(auth_header_str) => {
                    // Pull the token from the final part of the string 'Bearer <token>'
                    let token = auth_header_str.split_whitespace().last();

                    let token = match token {
                        Some(t) => t,
                        None => ""
                    };

                    // Decode the header of the JWT which contains the 'kid'
                    match decode_header(token) {
                        Ok(decoded_token) => {
                            let kid = match decoded_token.kid{
                                Some(k) => k,
                                None => String::new()
                            };

                            // Retrieve the JWKS
                            let jwks_url = format!("{}/.well-known/jwks.json", state.auth0_domain);
                            match Jwks::from_jwks_url(jwks_url).await{
                                Ok(jwks) => {
                                    // Grab the correct JWK based on the kid from the header
                                    match jwks.keys.get(&kid){
                                        Some(jwk) => {
                                            // Configure the token validation to use RS256 decoding, valiate the expiration time, and do not validate the audience (yet)
                                            let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
                                            validation.validate_exp = true;
                                            validation.validate_aud = false;
                                            
                                            // Decode the token body
                                            match decode::<Claims>(token, &jwk.decoding_key, &validation){
                                                Ok(token_data) => {
                                                    println!("Valid token!");

                                                    if state.auth0_audience != token_data.claims.aud{
                                                        println!("Invalid audience: {}!", token_data.claims.aud);
                                                        return Err(StatusCode::UNAUTHORIZED);
                                                    }

                                                    return Ok(next.run(request).await)
                                                },
                                                Err(e) => {
                                                    println!("Failed to decode token using decode key from jwk: {}!", e);
                                                    return Err(StatusCode::UNAUTHORIZED);
                                                }
                                            }
                                        },
                                        None => {
                                            println!("Failed to get JWK from JWKS!");
                                            return Err(StatusCode::UNAUTHORIZED);
                                        }
                                    }
                                },
                                Err(_) => {
                                    println!("Failed to fetch jwks!");
                                    return Err(StatusCode::UNAUTHORIZED);
                                }
                            }
                        },
                        Err(_) => {
                            println!("Failed to decode token header!");
                            return Err(StatusCode::UNAUTHORIZED);
                        }
                    }
                },
                Err(_) => {
                    println!("Auth header not formatted correctly!");
                    return Err(StatusCode::UNAUTHORIZED);        
                }
            }
        },
        None => {
            println!("No auth header found!");
            return Err(StatusCode::UNAUTHORIZED);
        }
    }
}