module Pages.Create exposing (Model, Msg, init, update, view)

import Html exposing (Html, a, button, div, input, text)
import Html.Attributes as A
import Html.Events as E



-- Model


type Model
    = CreatingUrl String
    | Submitting String


type Msg
    = ClickedCreateLink
    | ClickedCancel



-- init


init : ( Model, Cmd Msg )
init =
    ( CreatingUrl "", Cmd.none )



-- VIEW


view : Model -> Html Msg
view model =
    let
        ( disabled, url ) =
            inputAttributes model
    in
    div [ A.class "container" ]
        [ div [ A.class "twelve columns" ]
            [ input [ A.type_ "text", A.placeholder "Url to shorten", A.autofocus (not disabled), A.value url, A.disabled disabled, A.class "u-full-width" ] []
            ]
        , div [ A.class "twelve columns" ]
            [ div [ A.class "two columns" ]
                [ a [ A.href "/" ]
                    [ button []
                        [ text "Cancel"
                        ]
                    ]
                ]
            , div [ A.class "eight columns" ]
                [ div [ A.class "two columns" ]
                    [ button [ E.onClick ClickedCreateLink, A.disabled disabled ] [ text "Create redirection" ]
                    ]
                ]
            ]
        ]


inputAttributes : Model -> ( Bool, String )
inputAttributes model =
    case model of
        CreatingUrl url ->
            ( False, url )

        Submitting url ->
            ( True, url )



-- UPDATE


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case ( msg, model ) of
        ( ClickedCreateLink, CreatingUrl _ ) ->
            ( model, Cmd.none )

        ( ClickedCancel, _ ) ->
            ( model, Cmd.none )

        ( _, _ ) ->
            ( model, Cmd.none )
