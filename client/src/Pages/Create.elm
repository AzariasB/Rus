module Pages.Create exposing (ExternalMsg(..), Model, Msg(..), init, update, view)

import Html exposing (Html, a, button, div, input, text)
import Html.Attributes as A
import Html.Events as E
import Http
import Json.Decode exposing (Decoder, bool, field, map2, string)
import Session



-- Model


type Status
    = CreatingUrl String
    | SubmittingUrl String
    | Error String


type alias Model =
    { status : Status, session : Session.Data }


type InternalMsg
    = ClickedCreateLink
    | TypedLink String


type ExternalMsg
    = CreatedLink (Result Http.Error CreateResponse)


type Msg
    = Internal InternalMsg
    | External ExternalMsg



-- init


init : Session.Data -> ( Model, Cmd Msg )
init session =
    ( { status = CreatingUrl "", session = session }, Cmd.none )



-- VIEW


view : Model -> Html Msg
view model =
    let
        ( disabled, url ) =
            inputAttributes model
    in
    div [ A.class "container" ]
        [ div [ A.class "twelve columns" ]
            [ input
                [ A.type_ "text"
                , A.placeholder "Url to shorten"
                , A.autofocus (not disabled)
                , A.value url
                , A.disabled disabled
                , A.class "u-full-width"
                , E.onInput (TypedLink >> Internal)
                ]
                []
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
                    [ button [ E.onClick <| Internal ClickedCreateLink, A.disabled disabled ] [ text "Create redirection" ]
                    ]
                ]
            ]
        ]


inputAttributes : Model -> ( Bool, String )
inputAttributes model =
    case model.status of
        CreatingUrl url ->
            ( False, url )

        SubmittingUrl url ->
            ( True, url )

        Error url ->
            ( False, url )



-- UPDATE


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case ( msg, model.status ) of
        ( Internal ClickedCreateLink, CreatingUrl url ) ->
            ( model, Cmd.map External <| createLink url )

        ( External (CreatedLink _), _ ) ->
            ( model, Cmd.none )

        ( Internal (TypedLink data), _ ) ->
            ( { model | status = CreatingUrl data }, Cmd.none )

        ( _, _ ) ->
            ( model, Cmd.none )


createLink : String -> Cmd ExternalMsg
createLink model =
    Http.request
        { url = "/api/v1/redirections"
        , body = Http.stringBody "application/x-www-form-urlencoded" ("long_url=" ++ model)
        , expect = Http.expectJson CreatedLink createDecoder
        , method = "POST"
        , headers = []
        , timeout = Nothing
        , tracker = Nothing
        }


type alias CreateResponse =
    { error : Bool
    , message : String
    }


createDecoder : Decoder CreateResponse
createDecoder =
    map2 CreateResponse
        (field "error" bool)
        (field "message" string)
