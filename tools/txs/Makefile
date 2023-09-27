
test:
# RUST_MIN_STACK=104857600 cargo t -- smoke_gov_script --nocapture
# RUST_MIN_STACK=104857600 cargo t -- smoke_upgrade_single_step --nocapture
	RUST_MIN_STACK=104857600 cargo t -- smoke_upgrade_multiple_steps --nocapture

NEWB_ADDR = "cb03a83fcdaeed36355e2293b431b9f222c27c90afc73bade100d295d3a71622"
NEWB_MNEM = "donkey board prosper legend advice inch example there enough problem grit mutual mutual advance slush roof idle multiply trim host buzz mistake weird absurd"

ALICE_MNEM = "talent sunset lizard pill fame nuclear spy noodle basket okay critic grow sleep legend hurry pitch blanket clerk impose rough degree sock insane purse"

BOB_MNEM = "ring pumpkin cake build jungle cloth bronze aerobic mechanic baby love melt below sight cotton trophy inquiry sugar exhibit sure first match ten clarify"

transfer:
	MNEM=${ALICE_MNEM} cargo r transfer -t ${NEWB_ADDR} -a 3

register:
	MNEM=${NEWB_MNEM} cargo r validator register

bid:
	MNEM=${NEWB_MNEM} cargo r validator pof -e 1000 -b .005

vouch:
	MNEM=${ALICE_MNEM} cargo r validator vouch --vouch-for ${NEWB_ADDR}
	MNEM=${BOB_MNEM} cargo r validator vouch --vouch-for ${NEWB_ADDR}

unjail:
	MNEM=${ALICE_MNEM} cargo r validator jail -u ${NEWB_ADDR}
