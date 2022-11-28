module Pages.Error exposing (Model, Msg, init, view)

import Html exposing (Html)
import Html.Attributes as A


type alias Model =
    { message : String
    }


type alias Msg =
    ()


init : String -> ( Model, Cmd msg )
init message =
    ( { message = message }, Cmd.none )


view : Model -> Html msg
view { message } =
    Html.div [ A.class "container" ]
        [ Html.p []
            [ Html.text "Hmmm looks like there was an error"
            ]
        , Html.p
            []
            [ Html.text message
            ]
        ]
