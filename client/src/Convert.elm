module Convert exposing (httpErrorToString)

import Http exposing (Error)


httpErrorToString : Error -> String
httpErrorToString err =
    case err of
        Http.BadUrl url ->
            "Bad Url : " ++ url

        Http.Timeout ->
            "Timeout"

        Http.NetworkError ->
            "Network error"

        Http.BadStatus status ->
            "Bad status : " ++ String.fromInt status

        Http.BadBody body ->
            "Bad body : " ++ body
