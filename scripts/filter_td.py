import json

from openff.qcsubmit.results.filters import (
    ConnectivityFilter,
    RecordStatusFilter,
    UnperceivableStereoFilter,
    HydrogenBondFilter,
    ElementFilter,
)

from openff.qcsubmit.results import TorsionDriveResultCollection
from qcportal.models.records import RecordStatusEnum
from openff.qcsubmit.results.filters import ResultRecordFilter


class ChargeCheckFilter(ResultRecordFilter):
    def _filter_function(self, result, record, molecule) -> bool:
        from openff.toolkit.utils.exceptions import (
            UnassignedMoleculeChargeException,
        )

        # Some of the molecules fail charging with am1bccelf10 either
        # because of no bccs or failed conformer generation, sometimes it
        # cannot be captured with just the cmiles present in the record
        # metadata, so reading from file and checking it
        can_be_charged = True
        try:
            molecule.assign_partial_charges(
                partial_charge_method="am1bccelf10"
            )
        except (UnassignedMoleculeChargeException, ValueError):
            can_be_charged = False

        return can_be_charged


entries = dict(json.loads(r"""{json}"""))
dataset = TorsionDriveResultCollection(entries=entries)

# TODO pass in include_iodine
elements = ["H", "C", "N", "O", "S", "P", "F", "Cl", "Br"]

dataset = dataset.filter(
    HydrogenBondFilter(method="baker-hubbard"),
    RecordStatusFilter(status=RecordStatusEnum.complete),
    ConnectivityFilter(tolerance=1.2),
    UnperceivableStereoFilter(),
    ElementFilter(allowed_elements=elements),
    ChargeCheckFilter(),
)

print(dataset.json())
