# Assume actions are hierarchical: R < RW < RWC < RWCU
allow_model(actor, "read", resource) :=
    allow_model(actor, "R", resource)
	| allow_model(actor, "RW", resource)
	| allow_model(actor, "RWC", resource)
	| allow_model(actor, "RWCU", resource);

allow_model(actor, "write", resource) :=
    allow_model(actor, "RW", resource)
	| allow_model(actor, "RWC", resource)
	| allow_model(actor, "RWCU", resource);

allow_model(actor, "create", resource) :=
    allow_model(actor, "RWC", resource)
	| allow_model(actor, "RWCU", resource);

allow_model(actor, "unlink", resource) :=
    allow_model(actor, "RWCU", resource);

# Lookup role for user
role(user, role) := group in user.groups, group.id = role;

# Top-level rules
allow_model(actor, action, resource) :=
    allow_dhi_billing(actor, action, resource);

allow_dhi_billing(actor, action, resource) :=
    role(actor, role),
    allow_dhi_billing_by_role(role, action, resource);

## dhi_billing Rules

# user_access.dhi_group_receptionist
allow_dhi_billing_by_role("user_access.dhi_group_receptionist", "RWCU", resource) := 
	resource in [
		"model_dhi_bill",
		"model_dhi_bill_line",
		"model_dhi_bill_products_line",
		"model_dhi_bill_extra_charges",
		"model_dhi_invoice",
		"model_dhi_invoice_line",
		"model_dhi_invoice_payment",
		"model_dhi_insurance_bill",
		"model_dhi_insurance_bill_line",
		"model_dhi_payment",
		"model_dhi_payment_invoice_line",
		"model_dhi_payment_details_line",
		"model_dhi_payment_deposit_line",
		"model_dhi_payment_other_payee",
		"model_dhi_payment_adjustment",
		"model_dhi_legacy_employee"
	];
allow_dhi_billing_by_role("user_access.dhi_group_receptionist", "RWC", resource) := 
	resource = "model_dhi_generate_invoice";
allow_dhi_billing_by_role("user_access.dhi_group_receptionist", "RW", resource) := 
	resource in [
		"model_dhi_payment_adjust_config",
		"model_dhi_opd_config",
		"model_dhi_opd_config_line",
		"model_dhi_bill_master",
		"model_dhi_bill_detail",
		"model_dhi_payment_detail",
		"model_dhi_payment_fraction",
		"model_dhi_payment_multiple_bill"
	];
allow_dhi_billing_by_role("user_access.dhi_group_receptionist", "R", resource) := 
	resource in [
		"model_dhi_fee_schedule",
		"model_dhi_fee_schedule_line",
		"model_dhi_fee_schedule_line_variants",
		"model_dhi_extra_charge",
		"model_dhi_insurance_card"
	];

# user_access.dhi_group_billing
allow_dhi_billing_by_role("user_access.dhi_group_billing", "RWCU", resource) := 
	resource in [
		"model_dhi_bill",
		"model_dhi_bill_line",
		"model_dhi_bill_products_line",
		"model_dhi_bill_extra_charges",
		"model_dhi_payment_adjust_config",
		"model_dhi_opd_config",
		"model_dhi_opd_config_line",
		"model_dhi_invoice",
		"model_dhi_invoice_line",
		"model_dhi_invoice_payment",
		"model_dhi_insurance_bill",
		"model_dhi_insurance_bill_line",
		"model_dhi_extra_charge",
		"model_dhi_payment",
		"model_dhi_payment_invoice_line",
		"model_dhi_payment_details_line",
		"model_dhi_payment_deposit_line",
		"model_dhi_payment_other_payee",
		"model_dhi_payment_adjustment",
		"model_dhi_bill_master",
		"model_dhi_bill_detail",
		"model_dhi_payment_detail",
		"model_dhi_payment_fraction",
		"model_dhi_payment_multiple_bill",
		"model_dhi_legacy_employee"
	];
allow_dhi_billing_by_role("user_access.dhi_group_billing", "RWC", resource) := 
	resource in [
		"model_dhi_insurance_card",
		"model_dhi_generate_invoice"
	];
allow_dhi_billing_by_role("user_access.dhi_group_billing", "R", resource) := 
	resource in [
		"model_dhi_fee_schedule",
		"model_dhi_fee_schedule_line",
		"model_dhi_fee_schedule_line_variants"
	];

# user_access.dhi_group_nurse
allow_dhi_billing_by_role("user_access.dhi_group_nurse", "RWC", resource) := 
	resource in [
		"model_dhi_bill",
		"model_dhi_bill_line",
		"model_dhi_bill_products_line",
		"model_dhi_bill_extra_charges",
		"model_dhi_invoice",
		"model_dhi_invoice_line",
		"model_dhi_invoice_payment",
		"model_dhi_insurance_bill",
		"model_dhi_insurance_bill_line"
	];
allow_dhi_billing_by_role("user_access.dhi_group_nurse", "RW", resource) := 
	resource in [
		"model_dhi_bill_master",
		"model_dhi_bill_detail",
		"model_dhi_payment_fraction",
		"model_dhi_payment_multiple_bill"
	];
allow_dhi_billing_by_role("user_access.dhi_group_nurse", "R", resource) := 
	resource in [
		"model_dhi_fee_schedule",
		"model_dhi_fee_schedule_line",
		"model_dhi_fee_schedule_line_variants",
		"model_dhi_extra_charge",
		"model_dhi_insurance_card",
		"model_dhi_payment",
		"model_dhi_payment_invoice_line",
		"model_dhi_payment_details_line",
		"model_dhi_payment_deposit_line",
		"model_dhi_payment_other_payee",
		"model_dhi_payment_adjustment",
		"model_dhi_opd_config",
		"model_dhi_opd_config_line"
	];

# user_access.dhi_group_doctor
allow_dhi_billing_by_role("user_access.dhi_group_doctor", "RWC", resource) := 
	resource in [
		"model_dhi_bill",
		"model_dhi_bill_line",
		"model_dhi_bill_products_line",
		"model_dhi_bill_extra_charges",
		"model_dhi_invoice",
		"model_dhi_invoice_line",
		"model_dhi_invoice_payment",
		"model_dhi_insurance_bill",
		"model_dhi_insurance_bill_line"
	];
allow_dhi_billing_by_role("user_access.dhi_group_doctor", "RW", resource) := 
	resource in [
		"model_dhi_bill_master",
		"model_dhi_bill_detail",
		"model_dhi_payment_fraction",
		"model_dhi_payment_multiple_bill"
	];
allow_dhi_billing_by_role("user_access.dhi_group_doctor", "R", resource) := 
	resource in [
		"model_dhi_fee_schedule",
		"model_dhi_fee_schedule_line",
		"model_dhi_fee_schedule_line_variants",
		"model_dhi_extra_charge",
		"model_dhi_insurance_card",
		"model_dhi_payment",
		"model_dhi_payment_invoice_line",
		"model_dhi_payment_details_line",
		"model_dhi_payment_deposit_line",
		"model_dhi_payment_other_payee",
		"model_dhi_payment_adjustment",
		"model_dhi_opd_config",
		"model_dhi_opd_config_line"
	];

# user_access.dhi_group_inventory
allow_dhi_billing_by_role("user_access.dhi_group_inventory", "RW", resource) := 
	resource in [
		"model_dhi_bill_master",
		"model_dhi_bill_detail",
		"model_dhi_payment_fraction",
		"model_dhi_payment_multiple_bill"
	];
allow_dhi_billing_by_role("user_access.dhi_group_inventory", "R", resource) := 
	resource in [
		"model_dhi_bill",
		"model_dhi_bill_line",
		"model_dhi_bill_products_line",
		"model_dhi_bill_extra_charges",
		"model_dhi_invoice",
		"model_dhi_invoice_line",
		"model_dhi_invoice_payment",
		"model_dhi_insurance_bill",
		"model_dhi_insurance_bill_line",
		"model_dhi_fee_schedule",
		"model_dhi_fee_schedule_line",
		"model_dhi_fee_schedule_line_variants",
		"model_dhi_extra_charge",
		"model_dhi_insurance_card",
		"model_dhi_payment",
		"model_dhi_payment_invoice_line",
		"model_dhi_payment_details_line",
		"model_dhi_payment_deposit_line",
		"model_dhi_payment_other_payee",
		"model_dhi_payment_adjustment"
	];

# user_access.dhi_group_lab
allow_dhi_billing_by_role("user_access.dhi_group_lab", "RWC", resource) := 
	resource in [
		"model_dhi_bill",
		"model_dhi_bill_line",
		"model_dhi_bill_products_line",
		"model_dhi_bill_extra_charges"
	];
allow_dhi_billing_by_role("user_access.dhi_group_lab", "RW", resource) := 
	resource in [
		"model_dhi_bill_master",
		"model_dhi_bill_detail",
		"model_dhi_payment_fraction",
		"model_dhi_payment_multiple_bill"
	];
allow_dhi_billing_by_role("user_access.dhi_group_lab", "R", resource) := 
	resource in [
		"model_dhi_invoice",
		"model_dhi_invoice_line",
		"model_dhi_invoice_payment",
		"model_dhi_insurance_bill",
		"model_dhi_insurance_bill_line",
		"model_dhi_fee_schedule",
		"model_dhi_fee_schedule_line",
		"model_dhi_fee_schedule_line_variants",
		"model_dhi_extra_charge",
		"model_dhi_insurance_card",
		"model_dhi_payment",
		"model_dhi_payment_invoice_line",
		"model_dhi_payment_details_line",
		"model_dhi_payment_deposit_line",
		"model_dhi_payment_other_payee",
		"model_dhi_payment_adjustment",
		"model_dhi_opd_config",
		"model_dhi_opd_config_line"
	];

# user_access.dhi_group_imaging
allow_dhi_billing_by_role("user_access.dhi_group_imaging", "RWC", resource) := 
	resource in [
		"model_dhi_bill",
		"model_dhi_bill_line",
		"model_dhi_bill_products_line",
		"model_dhi_bill_extra_charges"
	];
allow_dhi_billing_by_role("user_access.dhi_group_imaging", "RW", resource) := 
	resource in [
		"model_dhi_bill_master",
		"model_dhi_bill_detail",
		"model_dhi_payment_fraction",
		"model_dhi_payment_multiple_bill"
	];
allow_dhi_billing_by_role("user_access.dhi_group_imaging", "R", resource) := 
	resource in [
		"model_dhi_invoice",
		"model_dhi_invoice_line",
		"model_dhi_invoice_payment",
		"model_dhi_insurance_bill",
		"model_dhi_insurance_bill_line",
		"model_dhi_fee_schedule",
		"model_dhi_fee_schedule_line",
		"model_dhi_fee_schedule_line_variants",
		"model_dhi_extra_charge",
		"model_dhi_insurance_card",
		"model_dhi_payment",
		"model_dhi_payment_invoice_line",
		"model_dhi_payment_details_line",
		"model_dhi_payment_deposit_line",
		"model_dhi_payment_other_payee",
		"model_dhi_payment_adjustment",
		"model_dhi_opd_config",
		"model_dhi_opd_config_line"
	];

# user_access.dhi_group_hr
allow_dhi_billing_by_role("user_access.dhi_group_hr", "RWCU", resource) := 
	resource in [
		"model_dhi_bill",
		"model_dhi_bill_line",
		"model_dhi_bill_products_line",
		"model_dhi_bill_extra_charges",
		"model_dhi_invoice",
		"model_dhi_invoice_line",
		"model_dhi_invoice_payment",
		"model_dhi_extra_charge",
		"model_dhi_insurance_bill",
		"model_dhi_insurance_bill_line",
		"model_dhi_insurance_card",
		"model_dhi_payment_invoice_line",
		"model_dhi_payment_details_line",
		"model_dhi_payment_deposit_line",
		"model_dhi_payment_other_payee",
		"model_dhi_payment_adjustment",
		"model_dhi_legacy_employee"
	];
allow_dhi_billing_by_role("user_access.dhi_group_hr", "RWC", resource) := 
	resource = "model_dhi_payment";
allow_dhi_billing_by_role("user_access.dhi_group_hr", "R", resource) := 
	resource in [
		"model_dhi_fee_schedule",
		"model_dhi_fee_schedule_line",
		"model_dhi_fee_schedule_line_variants",
		"model_dhi_payment",
		"model_dhi_payment_other_payee",
		"model_dhi_bill_master",
		"model_dhi_bill_detail",
		"model_dhi_payment_detail",
		"model_dhi_payment_fraction",
		"model_dhi_payment_multiple_bill"
	];

# user_access.dhi_group_administration
allow_dhi_billing_by_role("user_access.dhi_group_administration", "RWCU", resource) := 
	resource in [
		"model_dhi_bill",
		"model_dhi_bill_line",
		"model_dhi_bill_products_line",
		"model_dhi_bill_extra_charges",
		"model_dhi_payment_adjust_config",
		"model_dhi_opd_config",
		"model_dhi_opd_config_line",
		"model_dhi_invoice",
		"model_dhi_invoice_line",
		"model_dhi_invoice_payment",
		"model_dhi_insurance_bill",
		"model_dhi_insurance_bill_line",
		"model_dhi_fee_schedule",
		"model_dhi_fee_schedule_line",
		"model_dhi_fee_schedule_line_variants",
		"model_dhi_extra_charge",
		"model_dhi_insurance_card",
		"model_dhi_payment",
		"model_dhi_payment_invoice_line",
		"model_dhi_payment_details_line",
		"model_dhi_payment_deposit_line",
		"model_dhi_payment_other_payee",
		"model_dhi_payment_adjustment",
		"model_dhi_bill_master",
		"model_dhi_bill_detail",
		"model_dhi_payment_detail",
		"model_dhi_payment_fraction",
		"model_dhi_payment_multiple_bill",
		"model_dhi_generate_invoice",
		"model_dhi_legacy_employee",
		"model_dhi_billing_discount_config",
		"model_dhi_billing_discount_config_line"
	];

# user_access.dhi_group_super_admin
allow_dhi_billing_by_role("user_access.dhi_group_super_admin", "RWCU", resource) := 
	resource in [
		"model_dhi_bill",
		"model_dhi_bill_line",
		"model_dhi_bill_products_line",
		"model_dhi_bill_extra_charges",
		"model_dhi_payment_adjust_config",
		"model_dhi_opd_config",
		"model_dhi_opd_config_line",
		"model_dhi_invoice",
		"model_dhi_invoice_line",
		"model_dhi_invoice_payment",
		"model_dhi_insurance_bill",
		"model_dhi_insurance_bill_line",
		"model_dhi_fee_schedule",
		"model_dhi_fee_schedule_line",
		"model_dhi_fee_schedule_line_variants",
		"model_dhi_extra_charge",
		"model_dhi_insurance_card",
		"model_dhi_payment",
		"model_dhi_payment_invoice_line",
		"model_dhi_payment_details_line",
		"model_dhi_payment_deposit_line",
		"model_dhi_payment_other_payee",
		"model_dhi_payment_adjustment",
		"model_dhi_bill_master",
		"model_dhi_bill_detail",
		"model_dhi_payment_detail",
		"model_dhi_payment_fraction",
		"model_dhi_payment_multiple_bill",
		"model_dhi_generate_invoice",
		"model_dhi_legacy_employee",
		"model_dhi_billing_discount_config",
		"model_dhi_billing_discount_config_line"
	];
