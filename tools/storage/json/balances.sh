community=$(cat state_epoch_79_ver_33217173.795d.json   | jq '[.[] | select(.comm_wallet != null) | .balance.coin] | add')
total=$(cat state_epoch_79_ver_33217173.795d.json |  jq '[.[] | .balance.coin] | add' )
slow_wallet_total=$(cat state_epoch_79_ver_33217173.795d.json | jq '[.[] | select(.slow_wallet != null) | .balance.coin] | add' )
slow_wallet_unlocked=$(cat state_epoch_79_ver_33217173.795d.json | jq '[.[] | .slow_wallet.unlocked] | add' )
locked=$(echo "scale=2; $slow_wallet_total-$slow_wallet_unlocked " | bc)
community_pct=$(echo "scale=2; $community/$total * 100" | bc)
echo "locked=$locked"
echo "community=$community $community_pct"
echo "total=$total"
