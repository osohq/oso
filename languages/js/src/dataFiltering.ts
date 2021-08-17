export class Relationship {
    kind: string;
    otherType: string;
    myField: string;
    otherField: string;

    constructor(kind: string, otherType: string, myField: string, otherField: string) {
        this.kind = kind;
        this.otherType = otherType;
        this.myField = myField;
        this.otherField = otherField;
    }
}

export class Constraint {
    kind: string;
    field: string;
    value: any;

    constructor(kind: string, field: string, value: any) {
        this.kind = kind;
        this.field = field;
        this.value = value;
    }
}