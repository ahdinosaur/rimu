// This file was generated by lezer-generator. You probably shouldn't edit it.
import {LRParser} from "@lezer/lr"
import {newlines, indentation, listItemMarkers, trackIndent} from "./tokens.js"
export const parser = LRParser.deserialize({
  version: 14,
  states: "-hO]Q]OOP!TOPOOO]Q]O'#DgOOQV'#DV'#DVO!YQ_O'#C_O#`QaO'#CbO#mQXO'#CaOOQR'#DW'#DWO#rQZO'#C`O$rQXO'#CjO$|QXO'#CmO%WQXO'#CpO%cQXO'#CrO&YQaO'#DfO&dQXO'#CxOOQP'#Dj'#DjO&kQZO'#DTOOQR'#Df'#DfQOQXOOP&pOXO)C?XO'OQZO,5:ROOQV-E7T-E7TO]Q]O,58{OOQP'#Cb'#CbOOQR-E7U-E7UO'WQaO,59UO'_QaO'#DnO'lQXO,59_O'qQaO'#DkO(OQXO,59XO(TQXO'#CqO(YQXO'#DmO(bQXO,59[O&dQXO,59^OOQR'#De'#DeO(gQXO'#CvOOQP,59a,59aO(qQXO,59cO(vQXO,59cO&dQXO,59gO&dQXO,59gO&dQXO,59gO&dQXO,59gO&dQXO,59gO&dQXO,59gO&dQXO,59gOOQR,5:Q,5:QO)QQaO,59dO]Q]O,59oPOOP'#DU'#DUP&pOXO/'4sPOOO/'4s/'4sOOQV'#Dh'#DhOOQV1G/m1G/mOOQR1G.g1G.gOOQP1G.p1G.pO)kQXO'#DZO)vQXO,5:YOOQP1G.y1G.yO*OQXO'#DXO*`QXO,5:VOOQP1G.s1G.sO&dQXO,59]O*kQXO'#DYO*yQXO,5:XOOQP1G.v1G.vO+RQaO1G.xO+lQXO,59bOOQP1G.}1G.}O+qQaO1G.}O+{QXO1G.}O-ZQaO1G/RO-bQaO1G/RO.mQaO1G/RO.tQaO1G/RO0PQaO1G/RO0WQaO1G/RO0|QaO1G/RO'OQZO1G/ZPOOP-E7S-E7SPOOO49*_49*_OOQP,59u,59uOOQP-E7X-E7XO1jQaO,59sOOQP-E7V-E7VO1wQaO1G.wOOQP,59t,59tOOQP-E7W-E7WOOQP1G.|1G.|OOQP7+$i7+$iO2RQXO7+$iO2]QaO7+$iOOQR7+$u7+$uO2dQaO<<HTOOQP<<HT<<HTOOQPAN=oAN=o",
  stateData: "2p~O!WOSPOS!RPQ~OVTOWTOX_OY_OZ_O]XO`YOcZOm^On^O!P`O!UQO~O!RcO~O!UQO!ORX!QRX!TRXVRXWRX~O]!^X`!^Xp!^Xq!^Xr!^Xs!^Xt!^Xu!^Xv!^X!c!^X~O!P!^X!Q!^X!]UX~P!nO!]fO~OVgOWgO!OSX!QSX!TSX~OW_OX_OY_OZ_O]XO`YOcZOm^On^O~OVjO[!bP~P$TOV_O_!_P~P$TOVgOWgOb!aP~OhqO~O]sO`vOpwOqxOryOszOt{Ou|Ov}O!cuO~O!PrO!QrO~P%hOV_O~P$TO!S!QO~OP!RO!PrO!QrO!W!RO~O!Q!UO!T!UO~O[!XO~P%hO!`!YO[!^X[!bX~P!nO[![O~O!`!]O_!_X[!_X~P%hO_!_O~O!]!`O~O!`!aOb!aX~Ob!cO~OV_O[!_P~P$TOV!fO~OV_O!]!hO~P$TO!Pla!Qla[la_la!`la!]labla~P%hOV!sO[}X!`}X~O!`!YO[!ba~OV_O_{X!`{X[{X~P$TO!`!]O_!_a[!_a~OVgOWgOb|X!`|X~O!`!aOb!aa~O!Pfi!Qfi[fi_fi!`fi!]fibfi~P%hO[!zO~O_!{O!]!|O~P%hOV_O_!{O~P$TO]sO`vO!cuOqoiroisoitoiuoivoi!Poi!Qoi[oi_oi!`oi!]oiboi~Opoi~P,VOpwO~P,VO]sO`vOpwOqxO!cuOsoitoiuoivoi!Poi!Qoi[oi_oi!`oi!]oiboi~Oroi~P-iOryO~P-iO]sO`vOpwOqxOryOszO!cuOuoivoi!Poi!Qoi[oi_oi!`oi!]oiboi~Otoi~P.{Ot{O~P.{O]sO`vOpwOqxOryOszOt{Ou|O!cuO~Ovoi!Poi!Qoi[oi_oi!`oi!]oiboi~P0_O_{a!`{a[{a~P%hObei!`ei~P%hOV_O_#QO~P$TO_#QO~P%hO_#RO~P%hOXYVY~",
  goto: "'l!cPPP!d!d!j!qPPPPPPP!|PP!|PP!|#e!|#kP!|$S!|!|PP!|PPPPPPP!d$g$m$v%P%V%]PPPPPPPPP%c%l%x&PP&V'`P'f'iXaOQf!QZVOQWf!QYUOQWf!QTnZ!a{_OQXY^fqsvwxyz{|}!Q!]!`!h!|QoZR!x!a{[OQXY^fqsvwxyz{|}!Q!]!`!h!|st]il!P!d!g!i!j!k!l!m!n!o!u!w!}#PQ!ScR!q!SWSOQf!QReSWWOQf!QRhWQ!^lR!v!^Q!boR!y!bQ!ZjR!t!ZQ!O]Q!TcR!r!SQbOQdQQ!WfR!p!QZROQSf!QQ!VdR#O!pW]OQf!QQiXSlYsQ!P^Q!dqQ!gvQ!iwQ!jxQ!kyQ!lzQ!m{Q!n|Q!o}Q!u!]Q!w!`Q!}!hR#P!|QmYR!esRpZRkX",
  nodeNames: "⚠ LineComment Template ListBlock ObjectBlock ObjectBlockEntry ObjectKey Identifier String Null Boolean Number ) ( ParenthesizedExpression ] [ ListExpression } { ObjectExpression ObjectExpressionEntry FunctionExpression FunctionExpressionParams FatArrow CallExpression CallExpressionArgs GetExpression UnaryExpression ArithmeticOp LogicOp BinaryExpression ArithmeticOp ArithmeticOp CompareOp CompareOp CompareOp CompareOp CompareOp IndentedBlock",
  maxTerm: 65,
  context: trackIndent,
  nodeProps: [
    ["group", -3,3,4,39,"Block",-13,7,8,9,10,11,14,17,20,22,25,27,28,31,"Block Expression"],
    ["openedBy", 12,"(",15,"[",18,"{"],
    ["closedBy", 13,")",16,"]",19,"}"]
  ],
  skippedNodes: [0,1,40],
  repeatNodeCount: 6,
  tokenData: ".l~Rrpq#]qr#ers#rst%`uv%zvw&Pxy&[yz&az{%z{|&f|}&k}!O&p!O!P&w!P!Q%z!Q![&|![!]'g!^!_'l!_!`'y!`!a'l!c!}(X!}#O(j#P#Q(o#Q#R(t#R#S(X#T#Y(X#Y#Z(y#Z#b(X#b#c+Z#c#h(X#h#i-V#i#o(X#o#p.V#p#q.[#q#r.g~#bP!W~pq#]R#jPnP!_!`#mQ#rOsQ~#uVOr#rrs$[s#O#r#O#P$a#P;'S#r;'S;=`%Y<%lO#r~$aOW~~$dRO;'S#r;'S;=`$m;=`O#r~$pWOr#rrs$[s#O#r#O#P$a#P;'S#r;'S;=`%Y;=`<%l#r<%lO#r~%]P;=`<%l#r~%eTP~OY%`Z]%`^;'S%`;'S;=`%t<%lO%`~%wP;=`<%l%`~&POp~~&SPvw&V~&[Ou~~&aO]~~&fO[~Q&kOqQ~&pO!`~R&wOqQmP~&|O!c~~'RQZ~!O!P'X!Q![&|~'[P!Q!['_~'dPZ~!Q!['_~'lO!]~~'qPr~!_!`'t~'yOr~~'|Q!_!`#m!`!a(S~(XOh~~(^SV~!Q![(X!c!}(X#R#S(X#T#o(X~(oO`~~(tO_~~(yOt~~)OTV~!Q![(X!c!}(X#R#S(X#T#U)_#U#o(X~)dUV~!Q![(X!c!}(X#R#S(X#T#`(X#`#a)v#a#o(X~){UV~!Q![(X!c!}(X#R#S(X#T#g(X#g#h*_#h#o(X~*dUV~!Q![(X!c!}(X#R#S(X#T#X(X#X#Y*v#Y#o(X~*}SY~V~!Q![(X!c!}(X#R#S(X#T#o(X~+`UV~!Q![(X!c!}(X#R#S(X#T#i(X#i#j+r#j#o(X~+wUV~!Q![(X!c!}(X#R#S(X#T#`(X#`#a,Z#a#o(X~,`UV~!Q![(X!c!}(X#R#S(X#T#`(X#`#a,r#a#o(X~,ySX~V~!Q![(X!c!}(X#R#S(X#T#o(X~-[UV~!Q![(X!c!}(X#R#S(X#T#f(X#f#g-n#g#o(X~-sUV~!Q![(X!c!}(X#R#S(X#T#i(X#i#j*_#j#o(X~.[Oc~~._P#p#q.b~.gOv~~.lOb~",
  tokenizers: [newlines, indentation, listItemMarkers, 0, 1],
  topRules: {"Template":[0,2]},
  tokenPrec: 809
})
