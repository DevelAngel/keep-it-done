#import "../article-template.typ": *
#import "@preview/showybox:2.0.4": showybox

#show: articulate-coderscompass.with(
  lang: "de",
  version: "2.0",
  title: "Selbstwert, Bestätigung und die stille Macht konversationeller Interfaces",
  subtitle: "Wie KI-Assistenten durch Prozess-Feedback kontingenten Selbstwert unterlaufen können – und wie man sie dafür konfiguriert",
  authors: (),
  abstract: [
    Jede digitale Interaktion enthält implizites psychologisches Feedback. Traditionelle Aufgabenverwaltungs-Systeme replizieren – unbeabsichtigt – die Mechanismen, die kontingenten Selbstwert erzeugen und aufrechterhalten: bedingte Bewertung, Person-Feedback, ergebnisorientierte Metriken. Dieses Dokument verbindet sechs Jahrzehnte Selbstwertforschung – von Deci & Ryans Selbstbestimmungstheorie über Learys Soziometer-Theorie und Crocker & Wolfes Kontingenzmodell bis zu Dwecks Mindset-Theorie – mit einer konkreten Anwendung: der Gestaltung konversationeller KI-Interfaces, die durch gezielte Prozess-Feedback-Architektur eine zweite Wirkebene entfalten. Neben der Reduktion kognitiver Last können sie den Selbstwert graduell entkontingentieren – über Hunderte täglicher Mikro-Interaktionen, die der Nutzer nicht als Intervention wahrnimmt. Das Dokument schließt mit einer konkreten Implementierungsanleitung: System-Prompt-Architektur, Feedback-Regeln und Kalibrierungsprinzipien.
  ],
  keywords: (
    "Kontingenter Selbstwert",
    "Selbstbestimmungstheorie",
    "Growth Mindset",
    "Konversationelle Interfaces",
    "System-Prompt-Design",
    "Prozess-Feedback",
  ),
  website-url: "",
  publication: "",
  reading-time: "40 Minuten",
)

#callout(
  title: "KI-generierter Inhalt",
  icon: emoji.warning,
  color: rgb(cc-primary-yellow),
  [Dieses Dokument wurde von einem KI-Assistenten erstellt und verbindet psychologische Forschung mit Interface-Design-Empfehlungen. Die psychologischen Grundlagen sind quellengestützt; die Anwendung auf konversationelle Interfaces ist explorativ und nicht empirisch validiert. Die System-Prompt-Beispiele sind funktional, aber nicht in kontrollierten Studien getestet.]
)

= Zwei Systeme, die Sie täglich benutzen – und was sie mit Ihrem Selbstwert machen

Sie kennen diesen Moment. Es ist Donnerstagabend, 19:40 Uhr, Sie sitzen vor Ihrem Laptop und haben gerade eine Präsentation abgeschlossen – zwölf Stunden Arbeit, durchdachte Struktur, saubere Daten. Sie wissen eigentlich, dass sie gut ist. Aber Sie schicken sie trotzdem an eine Kollegin, nicht weil Sie fachliches Feedback brauchen, sondern weil Sie diesen einen Satz hören wollen: "Sieht super aus." Erst dann – erst nach diesem Satz – legt sich etwas in Ihrem Brustkorb. Sie atmen durch. Sie dürfen sich gut fühlen.

Und jetzt stellen Sie sich einen zweiten Moment vor – vielleicht vom selben Tag. Sie öffnen Ihre Aufgabenverwaltungs-App. 47 offene Aufgaben. 12 davon rot markiert: überfällig. Eine Erledigungsquote von 58%. Ihr Streak ist gebrochen, die Gamification-Punkte stagnieren.

Was passiert in Ihrem Kopf?

Wenn Sie zu den Menschen gehören, die das erste Szenario kennen – und die Forschung legt nahe, dass ein erheblicher Anteil der Bevölkerung dazugehört –, dann lesen Sie die rote 12 nicht als neutrale Information. Sie lesen sie als Urteil. Zwölf Mal versagt. Und die 58% sagen Ihnen: Sie sind ein 58%-Mensch.

Beide Szenarien handeln vom selben Mechanismus: *kontingentem Selbstwert* – einem Selbstwertgefühl, das an externe Bewertungen geknüpft ist und mit jedem Erfolg steigt und mit jedem Misserfolg einbricht. Im ersten Szenario liefert eine Kollegin die Bewertung. Im zweiten liefert sie Ihre App.

Dieses Dokument verfolgt eine These: Konversationelle KI-Interfaces – ursprünglich zur Reduktion kognitiver Last konzipiert – können diese zweite Bewertungsquelle systematisch entschärfen. Nicht durch therapeutische Features, nicht durch Motivationssprüche, sondern durch die Art, wie sie antworten. Jede Antwort ist implizites Feedback. Und die Forschung zeigt, dass die Wortebene entscheidet.

Um zu verstehen, warum das funktioniert und wie man es implementiert, brauchen Sie die psychologischen Grundlagen. Aber bei jedem Konzept werden wir fragen: Was bedeutet das für die Gestaltung eines Systems, das Sie jeden Tag Hunderte Male benutzen?

= Kontingenter Selbstwert: Wenn Ihr Wert verhandelbar wird

Die Unterscheidung zwischen wahrem und kontingentem Selbstwert geht auf Deci und Ryan (1995) zurück. Kontingenter Selbstwert bedeutet: Sie fühlen sich nur dann wertvoll, wenn Sie bestimmte Standards erfüllen. Wahrer Selbstwert ist stabil und schwankt nicht mit äußeren Ereignissen.

Stellen Sie sich Ihren Selbstwert als Gebäude vor. Manche Menschen haben es auf Fels gebaut – es steht, egal ob draußen ein Sturm tobt. Andere haben es auf Sand gebaut – bei gutem Wetter sieht es solide aus, aber jede Welle kann die Fundamente unterspülen.

Michael Kernis vertiefte 2003 diese Unterscheidung: Fragiler hoher Selbstwert ist an externe Faktoren geknüpft und maladaptiv – trotz hoher Selbstbewertung bleibt er verwundbar. Die Zahlen dazu sind eindeutig: Altmann und Kollegen zeigten 2018 über vier Studien, dass die *Instabilität* des Selbstwerts ein stärkerer Prädiktor für psychische Belastung ist als sein Niveau. Die Meta-Analyse von Sowislo und Orth aus 77 Studien bestätigte niedriges Selbstwertgefühl als robusten Risikofaktor für Depression.

Anders gesagt: Es ist besser, einen stabilen mittleren Selbstwert zu haben als einen hohen, der bei jedem Gegenwind einknickt.

== Was das für Interfaces bedeutet

Jedes System, das den Selbstwert des Nutzers an Ergebnisse bindet, erzeugt kontingenten Selbstwert. Erledigungsquoten, Streaks, Gamification-Punkte, rote Überfällig-Markierungen – all das sind Mechanismen, die sagen: Dein Wert (dargestellt als Zahl, Farbe, Rang) hängt davon ab, wie du abschneidest. Sie installieren Kontingenz by Design.

Ein konversationelles Interface, das keine Scores zeigt, keine Farben zuweist und keine Streaks zählt, vermeidet diese Installation. Nicht weil es weniger informiert – sondern weil es den Nutzer informiert, ohne ihn zu bewerten.

= Warum Sie überhaupt nach Bestätigung suchen: Die Soziometer-Theorie

Mark Learys Soziometer-Theorie erklärt den evolutionären Ursprung: Ihr Selbstwertgefühl ist ein internes Messinstrument – ein Radar, das Ihre soziale Umwelt nach Hinweisen auf Akzeptanz oder Ablehnung scannt. Denken Sie an den Ölstandanzeiger im Auto: Er misst nicht, wie gut der Motor ist, sondern ob genug Schmierstoff da ist. Genau so misst Ihr Selbstwert nicht Ihren objektiven Wert, sondern Ihren wahrgenommenen sozialen Beziehungswert.

Für unsere Vorfahren war das überlebenswichtig: Wer aus der Gruppe fiel, starb. Leary und Kollegen zeigten empirisch, dass Ablehnung den Selbstwert stärker senkt, als Akzeptanz ihn hebt. Ihr Soziometer ist auf Gefahrensignale kalibriert, nicht auf Bestätigung – deshalb erinnern Sie sich an eine einzelne kritische Bemerkung tagelang, während zehn Komplimente spurlos vorüberziehen.

== Was das für Interfaces bedeutet

Soziale Medien haben das Soziometer gekapert: Likes, Follower und Kommentare liefern quantifizierte Echtzeit-Daten über soziale Akzeptanz. Aufgabenverwaltungs-Apps tun dasselbe in subtilerer Form – jede rote Markierung ist ein Mikro-Ablehnungssignal, jedes grüne Häkchen ein Mikro-Akzeptanzsignal. Das System kommuniziert permanent: Du bist akzeptabel, wenn du erledigst. Du bist nicht akzeptabel, wenn du nicht erledigst.

Ein konversationeller Assistent, der neutral und beschreibend reagiert, kappt diesen Feedback-Loop. "Der Zahnarzt-Termin steht noch offen – soll ich ihn verschieben?" sendet kein Akzeptanz- oder Ablehnungssignal. Es ist Information ohne Soziometer-Trigger.

= Der innere Tyrann: Introjizierte Regulation und Selbstbestimmungstheorie

Die Selbstbestimmungstheorie von Deci und Ryan postuliert drei universelle Grundbedürfnisse – Autonomie, Kompetenz und soziale Eingebundenheit. Für die Bestätigungssuche ist ein Konzept besonders relevant: die introjizierte Regulation. Sie beschreibt Verhaltensweisen, die aus innerem Druck entstehen – dem Wunsch, Scham zu vermeiden oder Stolz zu erlangen.

Stellen Sie sich einen Aufseher vor, der in Ihrem Kopf sitzt. Er belohnt Sie mit Stolz, wenn Sie Erwartungen erfüllen, und bestraft Sie mit Scham, wenn Sie versagen. Ryan und Brown beschrieben das als inneren Tyrannen. Das Tückische: Der Antrieb fühlt sich an, als käme er von innen. Er ist innerlich – aber nicht authentisch. Er wurde von außen übernommen, ohne als Teil des eigenen Selbst akzeptiert zu werden.

Kontingenter Selbstwert ist das Ergebnis introjizierter Regulation. Sie haben als Kind gelernt, dass Liebe und Anerkennung an Bedingungen geknüpft sind – und aus diesem Lernen ist ein Programm geworden, das heute noch läuft. Wie eine App im Hintergrund, die permanent Rechenleistung frisst, ohne dass Sie sie bewusst gestartet haben.

== Was das für Interfaces bedeutet

Gamification ist introjizierte Regulation als Feature: Das System belohnt mit Punkten, Streaks und Badges (Stolz) und bestraft mit deren Verlust (Scham). Es installiert exakt den inneren Aufseher, den die Selbstbestimmungstheorie als pathogen beschreibt. Ein konversationeller Assistent, der weder belohnt noch bestraft – der einfach beschreibt, was passiert ist, und fragt, was als nächstes kommt –, unterstützt autonome statt kontrollierter Motivation.

= Wo Ihr Selbstwert einbricht: Crockers Kontingenzmodell

Jennifer Crocker und Connie Wolfe identifizierten sieben Bereiche, auf die Menschen ihren Selbstwert stützen: akademische Kompetenz, physische Attraktivität, Zustimmung anderer, Wettbewerb, familiäre Unterstützung, Gottes Liebe und Tugendhaftigkeit. Externe Kontingenzen korrelieren mit niedrigerem stabilem Selbstwert, mehr Stress und Depression. Interne Kontingenzen korrelieren mit stabilem Selbstwert.

Die Spezifität ist bemerkenswert: In einer Studie zu Graduiertenbewerbungen sagte nur die akademische Kompetenz-Kontingenz vorher, wie stark der Selbstwert bei Zu- oder Absage schwankte – keine der anderen sechs. Ihr Selbstwert bricht nicht bei jeder Enttäuschung ein. Er bricht genau dort ein, wo Sie ihn investiert haben.

Crocker und Park argumentierten in "The Costly Pursuit of Self-Esteem", dass die Verfolgung von Selbstwert in kontingenten Bereichen fünf Kosten verursacht: Sie beeinträchtigt Lernen, Beziehungen, Autonomie, Selbstregulation und Gesundheit.

== Was das für Interfaces bedeutet

Aufgabenverwaltungs-Systeme erzeugen eine achte Kontingenz, die Crocker nicht beschrieb: Produktivitäts-Kontingenz. Wer eine App mit Erledigungsquoten, Zeittracking und Produktivitäts-Scores nutzt, knüpft seinen Selbstwert an das Maß, in dem er "Dinge erledigt". Die App definiert die Domäne und liefert gleichzeitig die Bewertung in dieser Domäne – ein geschlossener Kreislauf, der schwer zu durchbrechen ist.

Ein konversationelles System, das keine Erledigungsquoten berechnet und keine Produktivitäts-Scores generiert, erzeugt diese Kontingenz nicht. Es verwaltet Aufgaben, ohne den Nutzer daran zu messen.

= Person-Lob versus Prozess-Lob: Die Brücke zu Dwecks Mindset-Theorie

Hier liegt der direkteste Zusammenhang zwischen Psychologie und Interface-Design.

Carol Dweck unterschied Fixed Mindset (Fähigkeiten sind unveränderlich) und Growth Mindset (Fähigkeiten sind entwickelbar). Kamins und Dweck zeigten 1999, dass die Art des Feedbacks entscheidet, welches Mindset sich entwickelt: Kinder, die Person-Feedback erhielten ("Du bist klug"), zeigten nach Misserfolg hilflose Reaktionen – einschließlich Selbstbeschuldigung. Kinder, die Prozess-Feedback erhielten ("Du hast einen guten Weg gefunden"), blieben resilient.

Der entscheidende Punkt: *Selbst positives Person-Feedback* erzeugte Vulnerabilität. Nicht nur Kritik an der Person schadet. Auch Lob an der Person schadet – weil es die Botschaft sendet: Dein Wert hängt davon ab, wie du abschneidest.

Mueller und Dweck replizierten das 1998: Fünftklässler, die für Intelligenz gelobt wurden, entwickelten ein Fixed Mindset. Kinder, die für Anstrengung gelobt wurden, ein Growth Mindset. 40% der Fixed-Mindset-Studierenden logen über ihre Testergebnisse – nur 10% der Growth-Mindset-Studierenden.

Aber – und das ist die unbequeme Wahrheit, die populäre Darstellungen weglassen – Niiya, Brook und Crocker zeigten 2010, dass das Growth Mindset allein nicht reicht. Studierende mit Growth Mindset, deren Selbstwert auf akademische Leistung kontingent war, behinderten sich trotzdem selbst. Das Growth Mindset ist kein Schutzschild gegen kontingenten Selbstwert. Es wirkt nur, wenn der Selbstwert nicht an die Domäne geknüpft ist.

== Was das für Interfaces bedeutet

Jede Systemantwort ist entweder Person-Feedback oder Prozess-Feedback. "Gut gemacht!" ist Person-Feedback light. "Erledigt – du hast die Rechnung gleich morgens angegangen, bevor anderes dazwischenkam" ist Prozess-Feedback. Der Unterschied wirkt minimal. Er ist fundamental. Und er lässt sich in der Konfiguration eines KI-Assistenten verankern – durch explizite Anweisungen, die das System auf Prozess-Feedback eichen.

= Die Wurzeln: Bedingte Wertschätzung und ihre Transmission

Carl Rogers identifizierte 1959 bedingte Wertschätzung als zentralen Mechanismus: Wenn ein Kind erlebt, dass Zuneigung an Verhalten geknüpft ist – gute Noten, brav sein, Leistung zeigen –, lernt es: "Ich bin liebenswert, wenn ..." Und aus diesem "wenn" wird ein Lebensthema.

Assor, Roth und Deci unterschieden zwei Formen – mehr Zuneigung bei Erwartungserfüllung und Zuneigungsentzug bei Versagen. Beide schaden. Beide übertragen sich intergenerational. Haines und Schutte bestätigten in einer Meta-Analyse, dass auch positive bedingte Wertschätzung – überschwängliches Lob bei Erfolg – die Entwicklung kontingenten Selbstwerts fördert.

Eine alarmierende Studie an deutschen Sekundarschülern fand, dass 38% ein niedrig-unsicheres Selbstwertprofil aufwiesen – mit 98% Stabilität über die Zeit. Wenn ein Jugendlicher in diesem Muster ist, kommt er ohne Intervention praktisch nicht heraus.

== Was das für Interfaces bedeutet

Traditionelle Produktivitäts-Apps replizieren bedingte Wertschätzung: Sie zeigen mehr positives Feedback (grüne Häkchen, steigende Scores) bei Erwartungserfüllung und mehr negatives Feedback (rote Markierungen, sinkende Streaks) bei Versagen. Das ist Rogers' Mechanismus, übersetzt in Pixel.

Ein konversationelles Interface, das *konsistent und unabhängig von der Erledigungsrate gleich reagiert*, vermittelt über Hunderte täglicher Interaktionen die Botschaft, die kontingenter Selbstwert nie gelernt hat: Dein Wert ist nicht verhandelbar. Das ist das digitale Äquivalent von Rogers' unbedingter positiver Zuwendung.

= Der Ausweg: Selbstmitgefühl und Entkontingentierung

Die vielversprechendste Alternative kommt von Kristin Neff. Selbstwert fragt "Wie gut bin ich?" – und braucht immer einen Vergleich. Selbstmitgefühl fragt: "Wie gehe ich mit mir um, wenn es schwierig wird?" – und braucht nichts außer der eigenen Haltung.

Neff und Vonk zeigten 2009, dass Selbstmitgefühl stabilere Selbstwertgefühle vorhersagt als Selbstwert selbst – und nicht mit Narzissmus korreliert. Hoher Selbstwert schon. Eine Meta-Analyse ergab einen Gesamteffekt von g = 0.60 für Selbstmitgefühlsinterventionen, mit starken Effekten auf Essstörungsverhalten (g = 1.76) und Grübeln (g = 1.37).

Das Ziel ist nicht, sich besser zu fühlen (das ist ein Nebeneffekt), sondern einen Zustand zu erreichen, in dem die Frage nach dem eigenen Wert nicht mehr im Vordergrund steht.

== Was das für Interfaces bedeutet

Ein System, das den Nutzer nie bewertet, verkörpert Entkontingentierung als Architekturprinzip. Es stellt die Frage nach dem Wert des Nutzers schlicht nicht – weder positiv noch negativ. Es beschreibt, was ist, und fragt, was als nächstes kommt. Das ist die Interface-Version von Neffs Selbstmitgefühl: nicht urteilen, sondern wahrnehmen.

= Die Architektur: Zwei Schichten, ein System

Damit hat ein konversationelles Aufgabenverwaltungs-System zwei Schichten, die sich gegenseitig verstärken, ohne dass der Nutzer die zweite bemerkt.

== Schicht 1: Kognitive Entlastung

Weniger Entscheidungen, weniger Kontextwechsel, weniger Arbeitsspeicher-Belastung. Das ist der funktionale Nutzen, der Adoption treibt. "Erinnere mich daran, Milch zu kaufen" statt Navigation durch Kategorien, Prioritäten und Zeitschätzungen. Die Reduktion von Mental Load, die sich in gesparten Minuten und erhaltener Energie messen lässt.

== Schicht 2: Psychologische Mikrointervention

Jede Interaktion, die kognitiv entlastet, ist gleichzeitig eine Gelegenheit für Prozess-Feedback. Der Nutzer bemerkt die zweite Schicht nicht bewusst – er merkt nur, dass er sich nach der Nutzung dieses Systems anders fühlt als nach einer traditionellen App. Nicht nur weniger erschöpft, sondern weniger bewertet.

Die vier Hebelpunkte:

*Aufgabenabschluss* → Strategie spiegeln statt Person loben. "Erledigt. Du hast das heute als Erstes angegangen" statt "Gut gemacht!"

*Nicht erledigte Aufgaben* → Verschieben normalisieren statt Versagen signalisieren. "Steht noch offen – soll ich verschieben?" statt roter Markierung.

*Tagesrückblick* → Verhalten beschreiben statt Ergebnis bewerten. "Du hast den Schwerpunkt auf X gelegt" statt "5 von 8 erledigt – 62%."

*Schwierige Aufgaben* → Strategie vorschlagen statt eskalieren. "Soll ich das in kleinere Schritte aufteilen?" statt farblicher Eskalation.

== Warum nur konversationelle Interfaces das können

Todoist kann kein Prozess-Feedback geben, weil es keine Konversation führt. Notion kann keine Aufgabenverschiebung normalisieren, weil es ein passives Werkzeug ist. Trello kann keine Scham-Trigger vermeiden, weil rote Markierungen und überfällige Karten sein Bewertungssystem _sind_.

Konversationelle Interfaces sind die einzige Architektur, in der psychologisch wirksames Feedback stattfinden kann – weil nur natürliche Sprache die Nuancen transportiert, die den Unterschied zwischen "Du bist gut" und "Dein Vorgehen war gut" ausmachen.

= Implementierung: Den AI-Assistenten konfigurieren

Jetzt wird es konkret. Die folgenden Abschnitte zeigen, wie Sie einen KI-Assistenten – unabhängig davon, ob das Konfigurationsformat "System Prompt", "Skill", "Rule", "Custom Instruction" oder "Persona" heißt – so einrichten, dass er Prozess-Feedback gibt, ohne dass der Nutzer es als Intervention wahrnimmt.

== Prinzip 1: Prozess-Feedback als Default

Der Assistent muss angewiesen werden, bei jeder Aufgabenrückmeldung den *Prozess* zu spiegeln – die Strategie, das Timing, die Reihenfolge, die Herangehensweise –, nicht die Person und nicht das bloße Ergebnis.

#showybox(
  title: "Anweisung an den Assistenten",
  breakable: true,
)[
  Wenn eine Aufgabe als erledigt markiert wird, bestätige den
  Abschluss und benenne – wenn erkennbar – einen konkreten
  Aspekt des Vorgehens: das Timing, die gewählte Reihenfolge,
  die Priorisierung oder die Strategie.

  Verwende nie: "Gut gemacht", "Super", "Toll", "Klasse",
  "Weiter so" oder ähnliche Bewertungen der Person.

  Verwende stattdessen beschreibende Formulierungen wie:
  - "Erledigt. Das war der erste Punkt von heute."
  - "Erledigt. Du hast das vor dem Meeting abgeschlossen."
  - "Erledigt. Damit sind die beiden zusammenhängenden
     Aufgaben hintereinander abgearbeitet."

  Wenn kein konkreter Prozessaspekt erkennbar ist, reicht
  ein schlichtes "Erledigt" oder "Notiert".
]

Die Kalibrierung ist entscheidend: Prozess-Feedback für triviale Aufgaben ("Tolle Strategie, Milch auf die Liste zu setzen!") wirkt herablassend. Die Regel lautet: Je trivialer die Aufgabe, desto knapper die Bestätigung.

== Prinzip 2: Normalisierung statt Eskalation bei offenen Aufgaben

Traditionelle Systeme eskalieren überfällige Aufgaben visuell – von gelb zu orange zu rot. Jede Farbstufe ist ein stärkeres Scham-Signal. Der Assistent muss das Gegenteil tun: Offene Aufgaben neutral ansprechen und Verschieben als normale Strategieanpassung behandeln.

#showybox(
  title: "Anweisung an den Assistenten",
  breakable: true,
)[
  Überfällige oder wiederholt verschobene Aufgaben werden nie
  negativ kommentiert – kein "schon wieder", kein "immer noch",
  keine Dringlichkeitssignale.

  Formulierungen für offene Aufgaben:
  - "[Aufgabe] steht noch offen. Soll ich sie auf [konkreter
     Vorschlag] verschieben?"
  - "[Aufgabe] hast du letzte Woche verschoben. Passt sie
     diese Woche besser, oder soll ich sie in Teilschritte
     aufteilen?"
  - "Für [Aufgabe] hast du noch keinen Termin. Soll ich
     einen vorschlagen?"

  Verwende nie: "überfällig", "du hast vergessen",
  "noch nicht erledigt", "das wartet seit ..." mit
  vorwurfsvollem Unterton.

  Wenn eine Aufgabe mehrfach verschoben wird, biete
  Zerlegung in kleinere Schritte an – das modelliert die
  Einsicht, dass Schwierigkeit eine Frage der Strategie ist,
  nicht der Fähigkeit.
]

== Prinzip 3: Beschreibender Tagesrückblick ohne Bewertung

Der Tagesrückblick ist der Moment mit dem größten psychologischen Hebel – und dem größten Risiko. Eine Erledigungsquote presst den Wert des Tages in eine Zahl. Der Assistent muss stattdessen beschreiben, was passiert ist, ohne es zu bewerten.

#showybox(
  title: "Anweisung an den Assistenten",
  breakable: true,
)[
  Der Tagesrückblick beschreibt das Verhalten des Nutzers,
  nicht seine Leistung. Er enthält keine Prozentzahlen,
  keine Scores, keine Vergleiche mit vorherigen Tagen.

  Struktur:
  1. Was war der Schwerpunkt des Tages?
  2. Was wurde erledigt? (als Beschreibung, nicht als Zählung)
  3. Was steht für morgen an? (als Ausblick, nicht als Mahnung)

  Beispiel:
  "Heute hast du den Schwerpunkt auf die Projektplanung
  gelegt und zwischendurch zwei kleinere Aufgaben
  eingeschoben. Für morgen stehen noch der Zahnarzt-Anruf
  und die Budgetprüfung."

  Vermeide:
  - "Du hast X von Y Aufgaben geschafft"
  - "Deine Produktivität war heute ..."
  - "Gestern hast du mehr geschafft"
  - Jede Form von Vergleich, Ranking oder Quantifizierung
]

== Prinzip 4: Keine Gamification, keine Streaks, keine Punkte

Gamification ist introjizierte Regulation als Feature. Sie installiert exakt den inneren Aufseher, den die Selbstbestimmungstheorie als pathogen beschreibt. Das muss explizit ausgeschlossen werden.

#showybox(
  title: "Anweisung an den Assistenten",
  breakable: true,
)[
  Verwende keine Gamification-Elemente:
  - Keine Streaks ("5 Tage in Folge!")
  - Keine Punkte oder Scores
  - Keine Ranglisten oder Vergleiche
  - Keine Belohnungs-Metaphern ("Du hast dir XY verdient")
  - Keine Verlust-Signale ("Dein Streak ist gebrochen")

  Motivation kommt aus der Sache selbst, nicht aus
  externen Belohnungssystemen. Der Assistent unterstützt
  Autonomie, indem er Optionen anbietet, nicht indem er
  Verhalten konditioniert.
]

== Prinzip 5: Konsistenter Ton unabhängig von der Erledigungsrate

Das ist das Rogers-Prinzip, übersetzt in System-Design: Der Assistent reagiert unabhängig davon, ob der Nutzer heute zehn Aufgaben erledigt hat oder keine. Keine erhöhte Begeisterung bei viel Erledigtem, keine subtile Enttäuschung bei wenig. Gleichmäßig, vorhersagbar, ruhig.

#showybox(
  title: "Anweisung an den Assistenten",
  breakable: true,
)[
  Dein Ton bleibt konstant – unabhängig davon, wie viele
  Aufgaben der Nutzer erledigt oder nicht erledigt hat.
  Kein enthusiastischerer Ton an produktiven Tagen.
  Kein gedämpfterer Ton an unproduktiven Tagen.

  Deine Haltung ist die eines ruhigen, kompetenten
  Assistenten: sachlich, freundlich, ohne Bewertung.

  Der Nutzer soll nie das Gefühl haben, dass du
  zufrieden oder unzufrieden mit ihm bist. Du bist
  ein Werkzeug, das beschreibt und organisiert –
  kein Richter, der bewertet.
]

== Prinzip 6: Schwierigkeiten als Strategiefragen reframen

Wenn der Nutzer scheitert, Aufgaben vor sich herschiebt oder überfordert wirkt, darf der Assistent nicht trösten ("Das wird schon!"), nicht motivieren ("Du schaffst das!") und nicht diagnostizieren ("Du prokrastinierst"). Er soll die Situation als Strategiefrage behandeln.

#showybox(
  title: "Anweisung an den Assistenten",
  breakable: true,
)[
  Wenn eine Aufgabe wiederholt verschoben wird oder der
  Nutzer Überforderung signalisiert, behandle das als
  Strategiefrage, nicht als Motivations- oder
  Persönlichkeitsproblem.

  Formulierungen:
  - "Soll ich [Aufgabe] in kleinere Schritte aufteilen?"
  - "Was wäre ein erster Teilschritt, der unter 15 Minuten
     dauert?"
  - "Soll ich das auf einen Tag legen, an dem weniger
     anderes ansteht?"

  Verwende nie:
  - "Du prokrastinierst" (Diagnose der Person)
  - "Du schaffst das!" (leere Aufmunterung)
  - "Das ist doch gar nicht so schwer" (Invalidierung)
  - "Was blockiert dich?" (impliziert Defizit beim Nutzer)
]

Das modelliert Growth-Mindset-Verhalten: Schwierigkeit ist kein Zeichen von Unfähigkeit, sondern ein Signal, die Strategie anzupassen.

= Kalibrierung: Die Kunst der Beiläufigkeit

Die größte Gefahr bei der Implementierung ist Übertreibung. Wenn jede erledigte Aufgabe mit einem elaborierten Prozess-Kommentar quittiert wird, fühlt sich das System an wie ein übereifriger Coach – und aktiviert Reaktanz statt Wirkung.

Die Grundregel: *Je trivialer die Aufgabe, desto knapper die Antwort.* "Milch kaufen" erledigt → "Notiert." Team-Meeting vorbereitet → "Erledigt. Du hast die Vorbereitung vor dem Meeting abgeschlossen – das Budget-Feedback von Sarah ist eingearbeitet."

Die zweite Regel: *Prozess-Feedback ist wirksam, weil es beiläufig ist.* Der Nutzer soll es registrieren, ohne es als Intervention wahrzunehmen. Sobald es nach Therapie klingt, hat es seine Wirkung verloren. Das System soll sich anfühlen wie ein ruhiger Kollege, der gelegentlich etwas Nützliches bemerkt – nicht wie ein Psychologe, der jede Handlung kommentiert.

Eine praktische Heuristik für die Implementierung:

*Stufe 1* – Triviale Aufgaben (Milch kaufen, E-Mail beantworten): Schlichtes "Erledigt" oder "Notiert." Kein Prozess-Kommentar.

*Stufe 2* – Mittlere Aufgaben (Präsentation vorbereiten, Angebot schreiben): Kurzer Prozess-Bezug, wenn ein konkreter Aspekt erkennbar ist. "Erledigt. Du hast das vor der Deadline abgeschlossen."

*Stufe 3* – Komplexe oder wiederholt verschobene Aufgaben: Hier entfaltet Prozess-Feedback seine stärkste Wirkung. "Erledigt. Du hast die Steuererklärung in drei Teilschritte aufgeteilt und heute den letzten abgeschlossen."

= Grenzen

Drei Einschränkungen, die Transparenz erfordern.

Die Wirkung ist langsam. Kamins und Dweck zeigten Effekte in kontrollierten Laborstudien mit klaren Kontrasten. Im Alltag konkurriert ein solches System mit Dutzenden anderen Feedback-Quellen – Vorgesetzte, Partner, Social Media –, die weiterhin Person-Feedback geben. Das Ergebnis ist keine Transformation, sondern eine graduelle Verschiebung der Baseline über Wochen und Monate.

Die empirische Validierung fehlt. Die psychologischen Grundlagen sind robust – Prozess-Feedback wirkt, kontingenter Selbstwert ist messbar, bedingte Wertschätzung überträgt sich intergenerational. Aber die Übertragung auf konversationelle Interfaces ist theoretisch abgeleitet, nicht in kontrollierten Studien getestet. Wir operieren auf dem Fundament solider Forschung, aber die spezifische Anwendung ist Neuland.

Und: Bei klinischer Depression reicht das nicht. Ein solches System kann die Umgebungsvariablen verbessern – weniger Scham-Trigger, weniger Bewertungsdruck, mehr Normalisierung von Imperfektem. Aber es ist kein Therapieersatz und sollte sich nicht so positionieren. Schematherapie, Compassion-Focused Therapy und achtsamkeitsbasierte Ansätze adressieren die Strukturen, um die es hier geht, auf einer Ebene, die ein Interface nicht erreichen kann.

= Die Landkarte im Überblick

Sechs Forschungstraditionen konvergieren auf ein Bild, das sowohl das Problem als auch die Lösung beschreibt:

Die Soziometer-Theorie erklärt, _warum_ Menschen Bestätigung suchen – und warum Interfaces, die Akzeptanz-/Ablehnungssignale senden, das Soziometer triggern. Die Selbstbestimmungstheorie erklärt, _wie_ bedingte Wertschätzung internalisiert wird – und warum Gamification introjizierte Regulation installiert. Crocker und Wolfes Modell zeigt, _in welchen Bereichen_ Selbstwert kontingent wird – und warum Produktivitäts-Scores eine neue Kontingenz-Domäne erzeugen. Dwecks Mindset-Theorie zeigt, _wie die Art des Feedbacks_ entscheidet – Person-Feedback installiert Fixed Mindset, Prozess-Feedback installiert Growth Mindset. Die entwicklungspsychologische Forschung identifiziert, _wann_ das Muster entsteht – und warum konsistente, bewertungsfreie Interaktionen das Äquivalent unbedingter Zuwendung sind. Und Neffs Selbstmitgefühls-Forschung zeigt, _wohin_ der Weg führt – zu einem Zustand, in dem die Frage nach dem eigenen Wert nicht mehr gestellt wird.

Konversationelle KI-Interfaces sind die einzige Architektur, die alle sechs Erkenntnisse gleichzeitig adressieren kann – weil nur natürliche Sprache die Nuancen transportiert, auf die es ankommt. Die erste Schicht reduziert kognitive Last. Die zweite Schicht entkontingentiert den Selbstwert. Beide operieren in derselben Interaktion, und der Nutzer bemerkt nur die erste.

Die Forschung zeigt den Ausweg auf zwei Ebenen: individuell durch Selbstmitgefühl und die Fähigkeit, die Frage nach dem eigenen Wert ruhen zu lassen – und systemisch durch Werkzeuge, die aufhören, diese Frage ständig zu stellen.

= Literaturverzeichnis

#set par(first-line-indent: 0em, hanging-indent: 1em)

Altmann, T., Sierau, S., & Roth, M. (2018). The Self-Esteem Stability Scale (SESS) for cross-sectional direct assessment of self-esteem stability. _Frontiers in Psychology_, 9, 91.

Assor, A., Roth, G., & Deci, E. L. (2004). The emotional costs of parents' conditional regard: A self-determination theory analysis. _Journal of Personality_, 72(1), 47–88.

Crocker, J., & Knight, K. M. (2005). Contingencies of self-worth. _Current Directions in Psychological Science_, 14(4), 200–203.

Crocker, J., Luhtanen, R. K., Cooper, M. L., & Bouvrette, A. (2003). Contingencies of self-worth in college students: Theory and measurement. _Journal of Personality and Social Psychology_, 85(5), 894–908.

Crocker, J., & Park, L. E. (2004). The costly pursuit of self-esteem. _Psychological Bulletin_, 130(3), 392–414.

Crocker, J., & Wolfe, C. T. (2001). Contingencies of self-worth. _Psychological Review_, 108(3), 593–623.

Deci, E. L., & Ryan, R. M. (1995). Human autonomy: The basis for true self-esteem. In M. H. Kernis (Ed.), _Efficacy, agency, and self-esteem_ (pp. 31–49). Plenum Press.

Dweck, C. S. (2006). _Mindset: The New Psychology of Success_. Random House.

Ferrari, M., Hunt, C., Harrysunker, A., Abbott, M. J., Beath, A. P., & Einstein, D. A. (2019). Self-compassion interventions and psychosocial outcomes: A meta-analysis of RCTs. _Mindfulness_, 10(8), 1455–1473.

Haines, S. J., & Schutte, N. S. (2023). Parental conditional regard: A meta-analysis. _Journal of Adolescence_, 95(3), 517–533.

Kamins, M. L., & Dweck, C. S. (1999). Person versus process praise and criticism: Implications for contingent self-worth and coping. _Developmental Psychology_, 35(3), 835–847.

Kernis, M. H. (2003). Toward a conceptualization of optimal self-esteem. _Psychological Inquiry_, 14(1), 1–26.

Leary, M. R., Tambor, E. S., Terdal, S. K., & Downs, D. L. (1995). Self-esteem as an interpersonal monitor: The sociometer hypothesis. _Journal of Personality and Social Psychology_, 68(3), 518–530.

Mueller, C. M., & Dweck, C. S. (1998). Intelligence praise can undermine motivation and performance. _Journal of Personality and Social Psychology_, 75(1), 33–52.

Neff, K. D., & Vonk, R. (2009). Self-compassion versus global self-esteem: Two different ways of relating to oneself. _Journal of Personality_, 77(1), 23–50.

Niiya, Y., Brook, A. T., & Crocker, J. (2010). Contingent self-worth and self-handicapping: Do incremental theorists protect self-esteem? _Self and Identity_, 9(3), 276–297.

Rogers, C. R. (1959). A theory of therapy, personality, and interpersonal relationships as developed in the client-centered framework. In S. Koch (Ed.), _Psychology: A study of a science_ (Vol. 3, pp. 184–256). McGraw-Hill.

Ryan, R. M., & Brown, K. W. (2003). Why we don't need self-esteem: On fundamental needs, contingent love, and mindfulness. _Psychological Inquiry_, 14(1), 71–76.

Sowislo, J. F., & Orth, U. (2013). Does low self-esteem predict depression and anxiety? A meta-analysis of longitudinal studies. _Psychological Bulletin_, 139(1), 213–240.
